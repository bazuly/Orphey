use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use tokio::time::Instant;

use crate::{
    models::{
        load::{LoadProfile, LoadReport, Sample},
        request::RequestDraft,
    },
    repository::http::build_request,
};

pub fn summarize(samples: Vec<Sample>, elapsed_ms: u64, profile: LoadProfile) -> LoadReport {
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

pub fn percentile(sorted: &[u64], percent: f64) -> u64 {
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

pub async fn window_worker_loop(
    from_ms: u64,
    until_ms: Option<u64>,
    total_vu: usize,
    draft: RequestDraft,
    base_url: String,
    client: reqwest::Client,
    next: Arc<AtomicUsize>,
    done: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<Sample>>>,
    started: Instant,
) {
    loop {
        if done.load(Ordering::Relaxed) {
            return;
        }
        let now = started.elapsed().as_millis() as u64;
        if now >= from_ms {
            break;
        }
        tokio::time::sleep(Duration::from_millis((from_ms - now).min(50))).await;
    }

    loop {
        if done.load(Ordering::Relaxed) {
            return;
        }
        if let Some(until) = until_ms {
            if started.elapsed().as_millis() as u64 >= until {
                return;
            }
        }
        let n = next.fetch_add(1, Ordering::Relaxed);
        if n >= total_vu {
            done.store(true, Ordering::Relaxed);
            return;
        }
        let sample = execute_single_request(&draft, &base_url, &client).await;
        samples.lock().expect("lock error").push(sample);
    }
}

pub fn window(
    index: usize,
    start: usize,
    max: usize,
    ramp_up_ms: u64,
    hold_ms: u64,
    ramp_down_ms: u64,
) -> (u64, Option<u64>) {
    if index < start || max <= start {
        return (0, None);
    }
    let steps = (max - start) as u64;
    let from_ms = ramp_up_ms * (index - start) as u64 / steps;
    let hold_ends = ramp_up_ms.saturating_add(hold_ms);
    let until_ms = hold_ends + ramp_down_ms * (max - index) as u64 / steps;
    (from_ms, Some(until_ms))
}
