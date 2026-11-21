use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use tokio::{task::JoinSet, time::Instant};

use crate::{
    error::Error,
    models::{
        load::{LoadReport, LoadSpec, Sample},
        request::RequestDraft,
    },
    repository::http::{self, build_request},
};

pub async fn run(spec: LoadSpec) -> Result<LoadReport, Error> {
    // TODO set in config in future or make it dynamic 1, 50
    let concurrency = spec.concurrency.clamp(1, 50);
    let total_vu = spec.total_vu.clamp(1, 2000);
    let client = http::create_client()?;

    let next = Arc::new(AtomicUsize::new(0));
    let samples = Arc::new(Mutex::new(Vec::with_capacity(total_vu)));
    let started = Instant::now();

    let mut tasks = spawn_workers(concurrency, total_vu, &spec, &client, &next, &samples);

    // waiting all tasks complete
    while tasks.join_next().await.is_some() {}

    let elapsed_ms = started.elapsed().as_millis() as u64;
    let transfer_samples = samples.lock().expect("lock error").clone();
    Ok(summarize(transfer_samples, elapsed_ms))
}

fn spawn_workers(
    concurrency: usize,
    total_vu: usize,
    spec: &LoadSpec,
    client: &reqwest::Client,
    next: &Arc<AtomicUsize>,
    samples: &Arc<Mutex<Vec<Sample>>>,
) -> JoinSet<()> {
    let mut tasks = JoinSet::new();

    for _ in 0..concurrency {
        tasks.spawn(worker_loop(
            total_vu,
            spec.draft.clone(),
            spec.base_url.clone(),
            client.clone(),
            next.clone(),
            samples.clone(),
        ));
    }
    tasks
}

async fn worker_loop(
    total_vu: usize,
    draft: RequestDraft,
    base_url: String,
    client: reqwest::Client,
    next: Arc<AtomicUsize>,
    samples: Arc<Mutex<Vec<Sample>>>,
) {
    loop {
        let n = next.fetch_add(1, Ordering::Relaxed);
        if n >= total_vu {
            break;
        }
        let sample = execute_single_request(&draft, &base_url, &client).await;
        samples.lock().expect("lock error").push(sample);
    }
}

async fn execute_single_request(
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

fn summarize(samples: Vec<Sample>, elapsed_ms: u64) -> LoadReport {
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
