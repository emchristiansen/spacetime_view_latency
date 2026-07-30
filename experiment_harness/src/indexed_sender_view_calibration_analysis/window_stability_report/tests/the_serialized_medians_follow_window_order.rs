//! Each emitted median is the median of *its own* window, on a series whose order sorting destroys.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;
use crate::indexed_sender_view_calibration_analysis::window_stability_report::WindowStabilityReport;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: the *association* between a position and its median, which a monotone fixture cannot
/// establish.
///
/// **The mutant this exists to kill.** `the_full_median_series_reaches_the_artifact` runs on a
/// strictly increasing series, so its window medians come out already ascending. A projection that
/// cloned the median vector, sorted it, and only then enumerated positions would reproduce that
/// fixture exactly and pass every one of its assertions — count, exact values, and positions alike —
/// while attaching medians to the wrong window starts on any real series. Checking every element
/// rules out truncation and duplication, but it cannot rule out a permutation the fixture is already
/// in.
///
/// So this test uses the symmetric V of `every_position_attaining_the_worst_deviation_is_retained`:
/// `1_000_000 + d`, where `d` counts `500 … 1` down to the midpoint and `1 … 500` back up. Its window
/// medians fall and then rise, so the window order is neither the ascending nor the descending sort
/// of itself — asserted below rather than assumed, because that inequality is the entire reason this
/// fixture discriminates where the other does not.
///
/// **The expectation comes from a different route than the code under test.** Each expected value is
/// `median` applied directly to that window's slice of the samples, rather than anything
/// [`WindowMedianSeries`] produced. A defect in the scan cannot propagate into the expectation and
/// cancel out.
#[test]
fn the_serialized_medians_follow_window_order() {
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
    let exact: Vec<Rational> = samples
        .iter()
        .map(|sample| Rational::from_int(*sample as i128))
        .collect();

    let width = CandidateWindow::W10.get();
    let expected: Vec<Rational> = (0..=(MAX_PACED_SAMPLES_USIZE - width))
        .map(|position| median(&exact[position..position + width]))
        .collect();

    // The property that makes this fixture discriminating: window order is not a sort of itself, in
    // either direction, so a projection that sorted before enumerating could not reproduce it.
    let mut ascending = expected.clone();
    ascending.sort();
    assert_ne!(
        ascending, expected,
        "the V's window medians must not already be in ascending order, or a sort-before-enumerate \
         projection would satisfy this test too"
    );
    let descending: Vec<Rational> = ascending.iter().rev().copied().collect();
    assert_ne!(
        descending, expected,
        "nor in descending order, which would let a reverse-sorting projection through"
    );

    let replicate = replicate(samples);
    let series = WindowMedianSeries::of(&replicate, CandidateWindow::W10);
    let rendered = serde_json::to_value(WindowStabilityReport::of(&series, median(&exact)))
        .expect("the report serializes");
    let emitted = rendered["medians"]
        .as_array()
        .expect("the full median series is a list");

    assert_eq!(
        emitted.len(),
        expected.len(),
        "one median per contiguous window, on this series as on any other"
    );
    for (position, value) in expected.iter().enumerate() {
        let numerator = i64::try_from(value.numerator()).expect("the fixture's numerators fit i64");
        let denominator =
            i64::try_from(value.denominator()).expect("a canonical denominator fits i64");
        assert_eq!(
            emitted[position],
            serde_json::json!({
                "position": position,
                "median": { "numerator": numerator, "denominator": denominator },
            }),
            "the median emitted at position {position} must be the median of the window starting \
             there, not whichever value a reordering left in that slot"
        );
    }
}

/// An admitted replicate carrying `samples`.
fn replicate(samples: Vec<u128>) -> CompleteReplicate {
    let line = LedgerFixture::complete(0).with_samples(samples).line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("the fixture line decodes")
        .into_lines();
    CompleteReplicate::admit(&lines[0].record).expect("the fixture line is admissible")
}
