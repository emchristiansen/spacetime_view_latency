//! A complete series cannot be sealed against a population verified for a different batch.

use super::fixture;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;
use crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries;

/// Coverage: the cross-check between the sample count and the count the composition was proven at.
///
/// Without it, an attempt could record a thousand samples, verify its caches against a state holding
/// only seven appends — a genuinely passing verification, against a genuinely wrong expectation — and
/// seal the result as complete. The row-by-row verifier cannot catch that on its own: it proves the
/// caches match *the expectation it was handed*, and an expectation built from the wrong append count
/// is internally consistent.
///
/// So three numbers must agree, not two: the samples recorded, the appends the composition was
/// verified against, and the frozen ceiling.
#[test]
fn a_complete_series_must_match_its_verified_appends() {
    let error = fixture::refusal(
        CalibrationSeries::recorded(
            fixture::samples(MAX_PACED_SAMPLES_USIZE),
            fixture::population(7),
        ),
        "a complete series verified against seven appends must not seal",
    );

    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("verified against all"),
        "the refusal must name the append disagreement, got {rendered:?}"
    );
}
