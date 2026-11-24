use crate::error::Error;

use crate::load::fixed::{self};
use crate::load::ramp_up::{self};
use crate::load::utils::summarize;
use crate::load::{peak, spike};
use crate::models::load::{LoadProfile, LoadReport, LoadSpec};

use crate::repository::http::{self};

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
        LoadProfile::Pike {
            start,
            max,
            ramp_up_ms,
            hold_ms,
            ramp_down_ms,
        } => {
            peak::run(
                &spec,
                &client,
                total_vu,
                start,
                max,
                ramp_up_ms,
                hold_ms,
                ramp_down_ms,
            )
            .await?
        }
    };
    let elapsed_ms = started.elapsed().as_millis() as u64;
    Ok(summarize(samples, elapsed_ms, spec.profile))
}
