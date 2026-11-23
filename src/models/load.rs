use serde::{Deserialize, Serialize};

use crate::models::request::RequestDraft;

#[derive(Debug, Clone)]
pub struct LoadSpec {
    pub draft: RequestDraft,
    pub base_url: String,
    pub total_vu: usize,
    pub profile: LoadProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LoadProfile {
    Fixed {
        concurrency: usize,
    },
    RampUp {
        start: usize,
        max: usize,
        ramp_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadReport {
    // pub profile: LoadProfile,
    pub total_vu: u64,
    pub counted_ok: u64,
    pub counted_failed: u64,
    pub elapsed_ms: u64,
    pub rps: f64,
    pub min_ms: u64,
    pub avg_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub statuses: Vec<(u16, u64)>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub status: Option<u16>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}
