//! Shared fixture: a vector of `count` distinct latency samples.

use std::time::Duration;

use crate::observation::latency_sample::LatencySample;

/// `count` distinct samples with increasing nanosecond values. `u64::try_from` rather than an `as`
/// cast so the fixture models the same no-silent-narrowing discipline as production code.
pub(super) fn samples(count: usize) -> Vec<LatencySample> {
    (0..count)
        .map(|i| {
            let nanos = u64::try_from(i).expect("test sample index fits u64");
            LatencySample::from_elapsed(Duration::from_nanos(nanos))
        })
        .collect()
}
