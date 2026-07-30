//! Both halves are equal, both medians are exact, and the difference is late minus early.

use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::trend_report::TrendReport;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: the segment sizes, the exact medians, and above all the **sign convention**.
///
/// The sign is the part a reader will get wrong if it is left implicit, so it is pinned here rather
/// than only documented. The fixture's second half is strictly slower than its first, and the
/// reported difference must therefore be **positive** — "positive means later samples ran slower".
/// A test asserting only the magnitude would pass under the opposite convention and leave every
/// future reader of the artifact guessing.
#[test]
fn the_split_is_exact_and_late_minus_early() {
    // First half constant at 1_000_000, second half constant at 3_000_000: both segment medians are
    // exact integers, and the drift is unmistakably later-is-slower.
    let half = MAX_PACED_SAMPLES_USIZE / 2;
    let samples: Vec<u128> = (0..MAX_PACED_SAMPLES_USIZE)
        .map(|index| if index < half { 1_000_000 } else { 3_000_000 })
        .collect();

    let rendered =
        serde_json::to_value(TrendReport::of(&replicate(samples))).expect("the report serializes");

    assert_eq!(
        rendered["early_samples"], half,
        "the split halves the series"
    );
    assert_eq!(rendered["late_samples"], MAX_PACED_SAMPLES_USIZE - half);
    assert_eq!(rendered["early_median"]["numerator"], 1_000_000);
    assert_eq!(rendered["early_median"]["denominator"], 1);
    assert_eq!(rendered["late_median"]["numerator"], 3_000_000);
    assert_eq!(
        rendered["difference"]["numerator"], 2_000_000,
        "the difference is late minus early, so a run that got slower reports a positive drift"
    );
    assert_eq!(rendered["difference"]["denominator"], 1);
}

/// An admitted replicate carrying `samples`.
fn replicate(samples: Vec<u128>) -> CompleteReplicate {
    let line = LedgerFixture::complete(0).with_samples(samples).line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("the fixture line decodes")
        .into_lines();
    CompleteReplicate::admit(&lines[0].record).expect("the fixture line is admissible")
}
