//! Every candidate width yields `n − W + 1` windows whose medians are exact and hand-checkable.

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// The first sample of the strictly increasing fixture series.
const BASE: i128 = 1_000_000;

/// Coverage: window count, exact median value, and position awareness, at every candidate width.
///
/// **The series is chosen so every median has a closed form.** With `sampleᵢ = BASE + i` strictly
/// increasing, the window starting at `p` of width `W` is an arithmetic run, so its median is exactly
/// `BASE + p + (W − 1)/2` — a value with a factor of two in it whenever `W` is even. That makes the
/// even case a real test of exactness rather than of rounding: `W = 10` at position 0 must be
/// `2000009/2`, not `1000004` or `1000005`.
///
/// `W = 1000` is covered by the same loop rather than special-cased, and is the boundary worth
/// naming: a complete series admits exactly one window of the widest candidate, so its median is the
/// full-series median.
#[test]
fn window_medians_are_exact_and_position_aware() {
    let replicate = increasing_replicate();

    for candidate in CandidateWindow::ALL {
        let width = candidate.get();
        let series = WindowMedianSeries::of(&replicate, candidate);

        assert_eq!(
            series.medians().len(),
            MAX_PACED_SAMPLES_USIZE - width + 1,
            "a width-{width} scan over the frozen series has exactly n − W + 1 windows"
        );

        for position in [0, series.medians().len() - 1] {
            // The exact median of the arithmetic run starting at `position`.
            let expected = Rational::new(
                2 * (BASE + position as i128) + width as i128 - 1,
                2,
            );
            assert_eq!(
                series.medians()[position], expected,
                "the width-{width} window at position {position} has an exact median, with no \
                 rounding on an even width"
            );
        }
    }

    let widest = WindowMedianSeries::of(&replicate, CandidateWindow::W1000);
    assert_eq!(
        widest.medians().len(),
        1,
        "the widest candidate admits exactly one window over a complete series"
    );
}

/// A replicate whose samples increase by one, so every window median has a closed form.
fn increasing_replicate() -> CompleteReplicate {
    let line = LedgerFixture::complete(0)
        .with_samples(
            (0..MAX_PACED_SAMPLES_USIZE)
                .map(|index| BASE as u128 + index as u128)
                .collect(),
        )
        .line();
    let records = parse_calibration_ndjson(&line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0]).expect("the fixture line is admissible")
}
