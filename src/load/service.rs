use std::collections::BTreeMap;

use tokio::time::Instant;

use crate::error::Error;

use crate::load::fixed::{self};
use crate::load::ramp_up::{self};
use crate::load::spike;
use crate::models::load::{LoadProfile, LoadReport, LoadSpec, Sample};
use crate::models::request::RequestDraft;
use crate::repository::http::{self, build_request};

pub async fn run(spec: LoadSpec) -> Result<LoadReport, Error> {
    let total_vu = spec.total_vu.clamp(1, 2000);
    let started = tokio::time::Instant::now();

    let client = http::create_client()?;
    let samples = match spec.profile.clone() {
        LoadProfile::Fixed { .. } => fixed::run(&spec, &client, total_vu).await?,
        LoadProfile::RampUp {
            start,
            max,
            ramp_ms,
        } => ramp_up::run(&spec, &client, total_vu, start, max, ramp_ms).await?,
        LoadProfile::Spike {
            base,
            spike,
            at_ms,
            for_ms,
        } => spike::run(&spec, &client, total_vu, base, spike, at_ms, for_ms).await?,
    };
    let elapsed_ms = started.elapsed().as_millis() as u64;
    Ok(summarize(samples, elapsed_ms, spec.profile))
}

fn summarize(samples: Vec<Sample>, elapsed_ms: u64, profile: LoadProfile) -> LoadReport {
    let total_vu = samples.len() as u64;
    let mut latencies = Vec::new();
    let mut statuses: BTreeMap<u16, u64> = BTreeMap::new();
    let mut errors = Vec::new();
    let mut counted_ok = 0;
    let mut counted_failed = 0;

    for sample in samples {
        latencies.push(sample.elapsed_ms);
        if let Some(code) = sample.status {
            *statuses.entry(code).or_default() += 1;
            if (200..300).contains(&code) {
                counted_ok += 1;
            } else {
                counted_failed += 1;
            }
        } else {
            counted_failed += 1;
        }
        if let Some(err) = sample.error {
            if errors.len() < 8 && !errors.contains(&err) {
                errors.push(err);
            }
        }
    }
    latencies.sort_unstable();
    let sum: u64 = latencies.iter().sum();
    let avg_ms = if total_vu == 0 { 0 } else { sum / total_vu };
    let seconds = (elapsed_ms as f64 / 1000.0).max(0.001);

    LoadReport {
        profile: profile,
        total_vu,
        counted_ok,
        counted_failed,
        elapsed_ms,
        rps: total_vu as f64 / seconds,
        min_ms: latencies.first().copied().unwrap_or(0),
        avg_ms,
        p95_ms: percentile(&latencies, 0.95),
        p99_ms: percentile(&latencies, 0.99),
        statuses: statuses.into_iter().collect(),
        errors,
    }
}

fn percentile(sorted: &[u64], percent: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * percent).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub async fn execute_single_request(
    draft: &RequestDraft,
    base_url: &str,
    client: &reqwest::Client,
) -> Sample {
    let began = Instant::now();
    let request = match build_request(draft.clone(), &base_url, &client) {
        Ok(req) => req,
        Err(err) => {
            return Sample {
                status: None,
                elapsed_ms: began.elapsed().as_millis() as u64,
                error: Some(err.to_string()),
            };
        }
    };

    match client.execute(request).await {
        Ok(response) => {
            let status = response.status().as_u16();
            let _ = response.bytes().await;
            Sample {
                status: Some(status),
                elapsed_ms: began.elapsed().as_millis() as u64,
                error: None,
            }
        }
        Err(err) => Sample {
            status: None,
            elapsed_ms: began.elapsed().as_millis() as u64,
            error: Some(err.to_string()),
        },
    }
}
