//! Extremes carry their positions, and a tie keeps every position rather than picking one.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;
use crate::indexed_sender_view_calibration_analysis::window_stability_report::WindowStabilityReport;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: min/max positions, the exact spread, and tie retention on the worst deviation.
///
/// **The tie case is the point.** A symmetric series deviates from its own full-series median by the
/// same amount at both ends, in opposite directions. A report that kept one representative position
/// would describe that as a single outlying window — the wrong story, and one a reader could not
/// recover from the artifact. Retaining every attaining position is what makes "two windows equally
/// far apart" distinguishable from "one window off".
#[test]
fn every_position_attaining_the_worst_deviation_is_retained() {
    // A symmetric V: the series descends to a midpoint and ascends back, so the first and last
    // windows are equally far from the full-series median, in the same direction.
    let half = MAX_PACED_SAMPLES_USIZE / 2;
    let samples: Vec<u128> = (0..MAX_PACED_SAMPLES_USIZE)
        .map(|index| {
            let distance = if index < half {
                half - index
            } else {
                index - half + 1
            };
            1_000_000 + distance as u128
        })
        .collect();

    let replicate = replicate(samples.clone());
    let full_series_median = median(
        &samples
            .iter()
            .map(|sample| Rational::from_int(*sample as i128))
            .collect::<Vec<_>>(),
    );
    let medians = WindowMedianSeries::of(&replicate, CandidateWindow::W10);
    let rendered = serde_json::to_value(WindowStabilityReport::of(&medians, full_series_median))
        .expect("the report serializes");

    assert_eq!(
        rendered["window_count"],
        MAX_PACED_SAMPLES_USIZE - CandidateWindow::W10.get() + 1,
        "the report states how many windows it summarised"
    );

    let positions = rendered["worst_deviation_positions"]
        .as_array()
        .expect("the attaining positions are a list");
    assert_eq!(
        positions.len(),
        2,
        "a symmetric series attains its worst deviation at both ends, and both are retained rather \
         than one being chosen as representative; got {positions:?}"
    );
    assert_eq!(
        positions[0].as_u64(),
        Some(0),
        "the attaining positions are ascending and include the leading window"
    );

    assert!(
        rendered["spread"]["numerator"].as_i64().expect("an integer") > 0,
        "a non-constant series has a strictly positive max-minus-min spread"
    );
    assert!(
        rendered["minimum"]["position"].as_u64().is_some()
            && rendered["maximum"]["position"].as_u64().is_some(),
        "both extremes carry the window position they were taken at"
    );
}

/// An admitted replicate carrying `samples`.
fn replicate(samples: Vec<u128>) -> CompleteReplicate {
    let line = LedgerFixture::complete(0).with_samples(samples).line();
    let records = parse_calibration_ndjson(&line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0]).expect("the fixture line is admissible")
}
