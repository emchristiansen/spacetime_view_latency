//! Extremes carry their positions, and a tie keeps every position rather than picking one.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;
use crate::indexed_sender_view_calibration_analysis::window_stability_report::WindowStabilityReport;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: min/max positions, the exact spread, and tie retention on the worst deviation.
///
/// **The tie case is the point.** A report that kept one representative position would describe a
/// plateau of equally extreme windows as a single outlying window — the wrong story, and one a reader
/// could not recover from the artifact.
///
/// # The series, and where its worst deviation actually falls
///
/// The samples form a symmetric V: `1_000_000 + d`, where `d` counts `500 … 1` down to the midpoint
/// and `1 … 500` back up. Every value from 1 to 500 therefore occurs exactly twice, so the sorted
/// series has 250 at position 500 and 251 at position 501, and the **full-series median is
/// `1_000_000 + 250.5`**.
///
/// The worst deviation is at the **trough**, not at the ends — which is why this test is written
/// against the derived positions rather than the intuition that a symmetric series must be extreme at
/// its extremities. The leading window `0..10` has median `1_000_000 + 495.5`, a deviation of 245.
/// The five windows centred on the trough see the multiset `{1,1,2,2,3,3,4,4,5,5}` and so have median
/// `1_000_000 + 3`, a deviation of 247.5 — strictly larger. Those five are `p = half−7 … half−3`:
/// window `p` covers `p … p+9`, and exactly these place five descending and five ascending samples
/// symmetrically about the turn.
///
/// A five-way tie is a stronger demonstration of the property than a two-way one, and asserting the
/// exact set proves the report neither truncated the plateau to one position nor widened it to
/// include the merely-near-extreme neighbours at `half−8` and `half−2`.
#[test]
fn every_position_attaining_the_worst_deviation_is_retained() {
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

    let positions: Vec<u64> = rendered["worst_deviation_positions"]
        .as_array()
        .expect("the attaining positions are a list")
        .iter()
        .map(|position| position.as_u64().expect("a window position is an integer"))
        .collect();
    let attaining: Vec<u64> = ((half - 7)..=(half - 3))
        .map(|position| position as u64)
        .collect();
    assert_eq!(
        positions, attaining,
        "every window attaining the worst deviation is retained, ascending, and none that does not \
         attain it is included"
    );

    assert!(
        rendered["spread"]["numerator"]
            .as_i64()
            .expect("an integer")
            > 0,
        "a non-constant series has a strictly positive max-minus-min spread"
    );
    assert_eq!(
        rendered["maximum"]["position"].as_u64(),
        Some(0),
        "the largest window median is the leading window, and the extreme carries its position"
    );
    assert_eq!(
        rendered["minimum"]["position"].as_u64(),
        Some(attaining[0]),
        "the smallest window median is the first trough window, reported at its own position rather \
         than at whichever position an iterator happened to reach last"
    );
}

/// An admitted replicate carrying `samples`.
fn replicate(samples: Vec<u128>) -> CompleteReplicate {
    let line = LedgerFixture::complete(0).with_samples(samples).line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("the fixture line decodes")
        .into_lines();
    CompleteReplicate::admit(&lines[0].record).expect("the fixture line is admissible")
}
