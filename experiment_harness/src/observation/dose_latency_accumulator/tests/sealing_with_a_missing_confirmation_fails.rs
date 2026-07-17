//! A batch that is one confirmation short cannot seal: the missing measured write is named, so a
//! partial dose never masquerades as a complete `RawLatencies`.

use std::time::Duration;

use crate::observation::dose_latency_accumulator::DoseLatencyAccumulator;
use crate::observation::latency_sample::LatencySample;
use crate::params::BATCH_SIZE_USIZE;

/// Fill every slot except index `0`, then seal: the accumulator is not complete and sealing fails,
/// naming the missing write rather than sealing a short batch.
#[test]
fn sealing_with_a_missing_confirmation_fails() {
    let mut accumulator = DoseLatencyAccumulator::new();
    for index in 1..BATCH_SIZE_USIZE {
        accumulator
            .record(
                index,
                LatencySample::from_elapsed(Duration::from_nanos(index as u64)),
            )
            .expect("recording a fresh in-range slot succeeds");
    }

    assert!(
        !accumulator.is_complete(),
        "a batch missing one confirmation is not complete"
    );
    let error = accumulator
        .seal()
        .expect_err("sealing a batch with a missing confirmation must fail");
    assert!(
        error.to_string().contains("missing confirmation"),
        "the error names the missing confirmation: {error}"
    );
}
