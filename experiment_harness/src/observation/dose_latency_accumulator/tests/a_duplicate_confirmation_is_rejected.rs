//! A second confirmation for an already-filled slot is a loud error, so a double-fire can never
//! overwrite a sample or inflate the filled count toward a false completion.

use std::time::Duration;

use crate::observation::dose_latency_accumulator::DoseLatencyAccumulator;
use crate::observation::latency_sample::LatencySample;

/// Recording index `0` twice fails the second time; the duplicate is named, and the first sample is
/// not silently replaced.
#[test]
fn a_duplicate_confirmation_is_rejected() {
    let mut accumulator = DoseLatencyAccumulator::new();
    accumulator
        .record(0, LatencySample::from_elapsed(Duration::from_nanos(7)))
        .expect("the first confirmation for a slot is accepted");

    let error = accumulator
        .record(0, LatencySample::from_elapsed(Duration::from_nanos(9)))
        .expect_err("a duplicate confirmation for the same slot must be rejected");
    assert!(
        error.to_string().contains("duplicate"),
        "the error names the duplicate condition: {error}"
    );
}
