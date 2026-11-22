use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use tokio::{task::JoinSet, time::Instant};

use crate::{
    error::Error,
    models::{
        load::{LoadProfile, LoadSpec, Sample},
        request::RequestDraft,
    },
    repository::http::build_request,
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
