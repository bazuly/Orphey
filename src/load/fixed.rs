use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use tokio::task::JoinSet;

use crate::{
    error::Error,
    load::service::execute_single_request,
    models::{
        load::{LoadProfile, LoadSpec, Sample},
        request::RequestDraft,
    },
};

pub(super) async fn run(
    spec: &LoadSpec,
    client: &reqwest::Client,
    total_vu: usize,
) -> Result<Vec<Sample>, Error> {
    let concurrency = if let LoadProfile::Fixed { concurrency } = spec.profile {
        concurrency.clamp(1, 50)
    } else {
        1
    };

    let next = Arc::new(AtomicUsize::new(0));
    let samples = Arc::new(Mutex::new(Vec::with_capacity(total_vu)));

    let mut tasks = spawn_workers(concurrency, total_vu, &spec, &client, &next, &samples);

    // waiting all tasks complete
    while tasks.join_next().await.is_some() {}

    let ready = samples.lock().expect("lock error").clone();
    Ok(ready)
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
