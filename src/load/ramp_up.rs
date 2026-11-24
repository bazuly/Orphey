use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::JoinSet;

use crate::load::utils::execute_single_request;
use crate::models::load::{LoadSpec, Sample};

use crate::error::Error;
use crate::models::request::RequestDraft;

pub(super) async fn run(
    spec: &LoadSpec,
    client: &reqwest::Client,
    total_vu: usize,
    start: usize,
    max: usize,
    ramp_ms: u64,
) -> Result<Vec<Sample>, Error> {
    let max = max.clamp(1, 50);
    let start = start.clamp(1, max);
    let ramp_ms = ramp_ms.min(60_000);

    let next = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::with_capacity(total_vu)));

    let mut tasks = spawn_workers(
        total_vu, spec, client, max, start, ramp_ms, &next, &samples, done,
    );

    while tasks.join_next().await.is_some() {}
    let ready = samples.lock().expect("lock error").clone();
    Ok(ready)
}

fn spawn_workers(
    total_vu: usize,
    spec: &LoadSpec,
    client: &reqwest::Client,
    max: usize,
    start: usize,
    ramp_ms: u64,
    next: &Arc<AtomicUsize>,
    samples: &Arc<Mutex<Vec<Sample>>>,
    done: Arc<AtomicBool>,
) -> JoinSet<()> {
    let mut tasks = JoinSet::new();

    for index in 0..max {
        tasks.spawn(worker_loop(
            delay_ms(index, start, max, ramp_ms),
            total_vu,
            spec.draft.clone(),
            spec.base_url.clone(),
            client.clone(),
            next.clone(),
            done.clone(),
            samples.clone(),
        ));
    }
    tasks
}

fn delay_ms(index: usize, start: usize, max: usize, ramp_ms: u64) -> u64 {
    if index < start || max <= start {
        return 0;
    }
    ramp_ms * (index - start) as u64 / (max - start) as u64
}

async fn worker_loop(
    delay_ms: u64,
    total_vu: usize,
    draft: RequestDraft,
    base_url: String,
    client: reqwest::Client,
    next: Arc<AtomicUsize>,
    done: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<Sample>>>,
) {
    let mut left = delay_ms;
    while left > 0 {
        if done.load(Ordering::Relaxed) {
            return;
        }
        let slice = left.min(50);
        tokio::time::sleep(Duration::from_millis(slice)).await;
        left -= slice;
    }

    loop {
        if done.load(Ordering::Relaxed) {
            return;
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
