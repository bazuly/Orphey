use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::JoinSet;
use tokio::time::Instant;

use crate::error::Error;
use crate::load::utils::execute_single_request;
use crate::models::load::{LoadSpec, Sample};
use crate::models::request::RequestDraft;

pub(super) async fn run(
    spec: &LoadSpec,
    client: &reqwest::Client,
    total_vu: usize,
    base: usize,
    spike: usize,
    at_ms: u64,
    for_ms: u64,
) -> Result<Vec<Sample>, Error> {
    let spike = spike.clamp(1, 50);
    let base = base.clamp(1, spike);
    let at_ms = at_ms.min(60_000);
    let until_ms = at_ms.saturating_add(for_ms.min(60_000));

    let next = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::with_capacity(total_vu)));
    let started = Instant::now();
    let mut tasks = JoinSet::new();

    for index in 0..spike {
        let (from_ms, until) = if index < base {
            (0, None)
        } else {
            (at_ms, Some(until_ms))
        };
        tasks.spawn(spike_worker_loop(
            from_ms,
            until,
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

async fn spike_worker_loop(
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
