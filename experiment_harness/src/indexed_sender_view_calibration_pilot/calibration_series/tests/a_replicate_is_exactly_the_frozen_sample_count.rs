//! A calibration replicate is the frozen count exactly — not at least it, and not near it.

use super::fixture;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;
use crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries;

/// Coverage: SSOT §569 evaluates every candidate `W` in `{10 … 1,000}` against **both complete
/// retained series**, so a short series cannot serve as a replicate at all — it cannot answer the
/// 1,000 case, and it cannot contribute to the between-attempt disagreement the rule weighs.
///
/// The short case is the one that matters and the one an earlier draft got wrong: `W_max` reads
/// naturally as a ceiling, and a constructor honouring only a ceiling would have sealed a
/// 400-sample attempt as a successful replicate. The long case is checked too, because a series
/// longer than the freeze is evidence from a method other than the one that was frozen.
#[test]
fn a_replicate_is_exactly_the_frozen_sample_count() {
    CalibrationSeries::recorded(
        fixture::samples(MAX_PACED_SAMPLES_USIZE),
        fixture::complete_population(),
    )
    .expect("the frozen count is a complete replicate");

    let one_short = MAX_PACED_SAMPLES_USIZE
        .checked_sub(1)
        .expect("the frozen sample count is positive");
    for short in [0, 1, one_short] {
        let error = fixture::refusal(
            CalibrationSeries::recorded(fixture::samples(short), fixture::complete_population()),
            "a short series is a failed attempt, never a replicate",
        );
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains("exactly the frozen"),
            "a short series must be refused for its length, got {rendered:?}"
        );
    }

    let one_long = MAX_PACED_SAMPLES_USIZE
        .checked_add(1)
        .expect("the frozen sample count plus one must not overflow usize");
    fixture::refusal(
        CalibrationSeries::recorded(fixture::samples(one_long), fixture::complete_population()),
        "a series longer than the freeze is not the method that was frozen",
    );
}
