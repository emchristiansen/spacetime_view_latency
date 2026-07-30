//! Disagreement covers the whole window-median series, and the extrema are ordinal, not signed.

use crate::indexed_sender_view_calibration_analysis::between_replicate_report::BetweenReplicateReport;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: the comparison domain, and the naming of the two extrema.
///
/// **The domain is the whole series.** An earlier draft compared only each replicate's *first*
/// window, which at `W = 10` discarded 990 of 991 available comparisons and rested on an unapproved
/// claim about what a future screen would sample. §569 asks about position-aware window medians in
/// both attempts, so every corresponding position is compared.
///
/// **The extrema are named ordinally, and that is tested with a uniformly-offset pair.** Here the
/// second replicate is slower at *every* position, so the maximum signed difference is positive and
/// the minimum is positive too. Under the earlier names — "greatest positive" and "greatest
/// negative" — the second field would have been asserting a sign this perfectly ordinary case does
/// not have.
#[test]
fn disagreement_is_measured_at_every_corresponding_position() {
    let first = replicate(0, 1_000_000);
    let second = replicate(1, 1_000_500);

    let candidate = CandidateWindow::W10;
    let rendered = serde_json::to_value(BetweenReplicateReport::of(
        &WindowMedianSeries::of(&first, candidate),
        &WindowMedianSeries::of(&second, candidate),
    ))
    .expect("the report serializes");

    assert_eq!(
        rendered["window_count"],
        MAX_PACED_SAMPLES_USIZE - candidate.get() + 1,
        "every corresponding window position is compared, not a single representative one"
    );

    // A uniform offset of 500 makes every per-position difference exactly 500.
    for field in [
        "maximum_signed_difference",
        "minimum_signed_difference",
        "greatest_absolute_difference",
    ] {
        assert_eq!(
            rendered[field]["difference"]["numerator"], 500,
            "{field} is 500 when the second replicate is uniformly 500 slower"
        );
    }
    assert_eq!(
        rendered["median_signed_difference"]["numerator"], 500,
        "the exact central summary of a uniform offset is that offset"
    );

    let positions = rendered["minimum_signed_difference"]["positions"]
        .as_array()
        .expect("attaining positions are a list");
    assert_eq!(
        positions.len(),
        MAX_PACED_SAMPLES_USIZE - candidate.get() + 1,
        "when every position ties, every position is retained rather than one being chosen"
    );
}

/// An admitted replicate whose samples increase by one from `base`.
fn replicate(ordinal: u32, base: u128) -> CompleteReplicate {
    let line = LedgerFixture::complete(ordinal)
        .with_samples(
            (0..MAX_PACED_SAMPLES_USIZE)
                .map(|index| base + index as u128)
                .collect(),
        )
        .line();
    let records = parse_calibration_ndjson(&line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0]).expect("the fixture line is admissible")
}
