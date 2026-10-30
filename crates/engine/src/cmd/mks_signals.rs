//! Bounded normalized motion sources. Evaluation is shared by the command and UI preview.
use super::CommandSpec;
use crate::{
    Result, Session,
    params::bad,
    schema::{TypedContract, output_schema_for},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Params {
    profile: String,
    duration_s: f64,
    ramp_s: f64,
    #[serde(default = "frequency")]
    frequency_hz: f64,
    #[serde(default = "samples")]
    samples: usize,
}
fn frequency() -> f64 {
    1.
}
fn samples() -> usize {
    121
}
#[derive(Serialize, JsonSchema)]
struct Output {
    profile: String,
    points: Vec<[f64; 2]>,
    normalized: bool,
}
const CONTRACT: TypedContract<Params> = TypedContract::new();
pub static COMMANDS: &[CommandSpec] =
    &[CommandSpec::new("mks.signal.sample", "Sample MKS Motion Source", crate::checked!("mks.signal.sample", sample))
        .icon("graph")
        .contract(&CONTRACT)
        .output(output_schema_for::<Output>)
        .noundo()
        .doc_only()];
pub(super) fn valid(profile: &str) -> bool {
    ["cosine", "quintic", "sine"].contains(&profile)
}
fn sample(_: &mut Session, p: &Value) -> Result<Value> {
    let p: Params = serde_json::from_value(p.clone()).map_err(|e| bad("mks.signal.sample", e.to_string()))?;
    if !valid(&p.profile)
        || !p.duration_s.is_finite()
        || !(0.001..=10.).contains(&p.duration_s)
        || !p.ramp_s.is_finite()
        || !(0.000001..=p.duration_s).contains(&p.ramp_s)
        || !p.frequency_hz.is_finite()
        || !(0.01..=20.).contains(&p.frequency_hz)
        || !(2..=1001).contains(&p.samples)
    {
        return Err(bad("mks.signal.sample", "cosine/quintic/sine; duration .001..10, ramp >0 within duration, frequency .01..20, samples 2..1001"));
    }
    let points: Vec<_> = (0..p.samples)
        .map(|i| {
            let t = p.duration_s * i as f64 / (p.samples - 1) as f64;
            let u = (t / p.ramp_s).clamp(0., 1.);
            let s = u * u * u * (10. + u * (-15. + 6. * u));
            let value = match p.profile.as_str() {
                "quintic" => s,
                "sine" => s * (2. * std::f64::consts::PI * p.frequency_hz * t).sin(),
                _ => 0.5 * (1. - (std::f64::consts::PI * u).cos()),
            };
            [t, value]
        })
        .collect();
    Ok(json!({"profile":p.profile,"points":points,"normalized":true}))
}
