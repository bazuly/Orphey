use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Mutex};

use tokio::task::JoinSet;
use tokio::time::Instant;

use crate::error::Error;
use crate::load::utils::{window, window_worker_loop};
use crate::models::load::{LoadSpec, Sample};

pub(super) async fn run(
    spec: &LoadSpec,
    client: &reqwest::Client,
    total_vu: usize,
    start: usize,
    max: usize,
    ramp_up_ms: u64,
    hold_ms: u64,
    ramp_down_ms: u64,
) -> Result<Vec<Sample>, Error> {
    let max = max.clamp(1, 50);
    let start = start.clamp(1, max);
    let ramp_up_ms = ramp_up_ms.min(60_000);
    let hold_ms = hold_ms.min(60_000);
    let ramp_down_ms = ramp_down_ms.min(60_000);
    let next = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::with_capacity(total_vu)));
    let started = Instant::now();
    let mut tasks = JoinSet::new();

    for index in 0..max {
        let (from_ms, until_ms) = window(index, start, max, ramp_up_ms, hold_ms, ramp_down_ms);
        tasks.spawn(window_worker_loop(
            from_ms,
            until_ms,
            total_vu,
            spec.draft.clone(),
            spec.base_url.clone(),
            client.clone(),
            next.clone(),
            done.clone(),
            samples.clone(),
            started,
        ));
    }
    while tasks.join_next().await.is_some() {}
    let ready = samples.lock().expect("lock error").clone();
    Ok(ready)
}
