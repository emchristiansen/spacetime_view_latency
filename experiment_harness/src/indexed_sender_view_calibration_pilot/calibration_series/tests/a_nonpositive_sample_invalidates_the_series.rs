//! One nonpositive sample invalidates the whole series rather than counting as flat.

use super::fixture;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;
use crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries;
use crate::indexed_sender_view_calibration_pilot::paced_sample_nanos::PacedSampleNanos;

/// Coverage: the spec invalidates a cell on a missing, non-finite, or nonpositive statistic rather
/// than treating it as a zero-latency observation.
///
/// A zero would otherwise be the most damaging value the ledger could carry: it is a *usable* number,
/// so every window median computed over the series would silently shift toward it, and the resulting
/// `W` would be chosen from a distribution that never happened.
///
/// Checked at the first, a middle, and the last position, because a scan that stopped early or
/// started late would pass one of them.
#[test]
fn a_nonpositive_sample_invalidates_the_series() {
    let last = MAX_PACED_SAMPLES_USIZE
        .checked_sub(1)
        .expect("the frozen sample count is positive");
    let middle = MAX_PACED_SAMPLES_USIZE
        .checked_div(2)
        .expect("two is not zero");

    for position in [0, middle, last] {
        let mut samples = fixture::samples(MAX_PACED_SAMPLES_USIZE);
        samples[position] = PacedSampleNanos::of(0);

        let error = fixture::refusal(
            CalibrationSeries::recorded(samples, fixture::complete_population()),
            "a nonpositive sample must invalidate its own attempt",
        );
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains(&format!("sample {position}")),
            "the refusal must name the offending position, got {rendered:?}"
        );
    }
}
