//! Every window median is emitted, at its own position, exactly — and `W = 1000` is the centre itself.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;
use crate::indexed_sender_view_calibration_analysis::window_stability_report::WindowStabilityReport;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// The first sample of the strictly increasing fixture series.
const BASE: i128 = 1_000_000;

/// Coverage: §615's full position-aware median series in the *emitted artifact*, at every candidate.
///
/// **This is asserted against the serialized value, not against `WindowMedianSeries`.** That the
/// analysis domain computes the whole series is already covered by
/// `window_medians_are_exact_and_position_aware`; what §615 requires is that the whole series
/// *reaches the report*, and a projection that quietly kept only the extrema would satisfy the
/// domain-level test unchanged. So every claim here is read back out of `serde_json::Value`.
///
/// **Every element is checked, not a sample of them.** A projection that truncated, deduplicated, or
/// reordered in the middle of the series would survive spot checks at the ends. Each element is
/// compared against a whole expected object, which additionally pins that an element carries exactly
/// its position and its median and nothing else.
///
/// # Why these medians are exactly hand-derivable
///
/// With `sampleᵢ = BASE + i` strictly increasing, the width-`W` window at position `p` is an
/// arithmetic run from `BASE + p` to `BASE + p + W − 1`, so its median is the midpoint of its two
/// ends: exactly `(2·(BASE + p) + W − 1) / 2`. Every candidate width is even, so that numerator is
/// **odd** at every position of every candidate — the fraction is already in lowest terms, the
/// denominator is `2` and never reduces to `1`, and each element is a genuine half-integer. Any
/// rounding anywhere on the path from samples to JSON would therefore have to show up as a
/// denominator of `1`; exactness here is observable rather than asserted.
///
/// # The `W = 1000` identity
///
/// A complete series admits exactly one window of the widest candidate, so that window *is* the whole
/// series and its median *is* the full-series median — the same value every other candidate's
/// deviations are measured against. It is checked twice over: once by value, against the independently
/// computed centre rather than against the closed form used above, and once through the report's own
/// arithmetic, where a window median identical to the centre must produce an exactly zero deviation
/// and an exactly zero spread. Passing by value while failing through the arithmetic would mean the
/// deviation is computed against something other than the centre it claims.
#[test]
fn the_full_median_series_reaches_the_artifact() {
    let samples: Vec<u128> = (0..MAX_PACED_SAMPLES_USIZE)
        .map(|index| BASE as u128 + index as u128)
        .collect();
    let replicate = increasing_replicate(&samples);
    let full_series_median = median(
        &samples
            .iter()
            .map(|sample| Rational::from_int(*sample as i128))
            .collect::<Vec<_>>(),
    );

    for candidate in CandidateWindow::ALL {
        let width = candidate.get();
        let series = WindowMedianSeries::of(&replicate, candidate);
        let rendered = serde_json::to_value(WindowStabilityReport::of(&series, full_series_median))
            .expect("the report serializes");

        let medians = rendered["medians"]
            .as_array()
            .expect("the full median series is a list");
        let expected_count = MAX_PACED_SAMPLES_USIZE - width + 1;
        assert_eq!(
            medians.len(),
            expected_count,
            "a width-{width} scan emits one median per contiguous window, so exactly n − W + 1 of \
             them reach the artifact"
        );
        assert_eq!(
            rendered["window_count"].as_u64(),
            Some(expected_count as u64),
            "the stated window count is the length of the series actually emitted, not a separately \
             computed number that could disagree with it"
        );

        for (index, element) in medians.iter().enumerate() {
            let expected_numerator = i64::try_from(2 * (BASE + index as i128) + width as i128 - 1)
                .expect("the fixture's exact numerators fit i64");
            assert_eq!(
                *element,
                serde_json::json!({
                    "position": index,
                    "median": { "numerator": expected_numerator, "denominator": 2 },
                }),
                "the width-{width} window at position {index} is emitted at its own position with \
                 its exact median, in lowest terms and without rounding"
            );
        }
    }

    let widest = WindowMedianSeries::of(&replicate, CandidateWindow::W1000);
    let rendered = serde_json::to_value(WindowStabilityReport::of(&widest, full_series_median))
        .expect("the report serializes");
    let expected_numerator =
        i64::try_from(full_series_median.numerator()).expect("the fixture's centre fits i64");
    let expected_denominator =
        i64::try_from(full_series_median.denominator()).expect("a canonical denominator fits i64");
    assert_eq!(
        *rendered["medians"]
            .as_array()
            .expect("the full median series is a list"),
        vec![serde_json::json!({
            "position": 0,
            "median": {
                "numerator": expected_numerator,
                "denominator": expected_denominator,
            },
        })],
        "the widest candidate emits exactly one window, at position 0, whose median is the \
         full-series median itself"
    );

    let zero = serde_json::json!({ "numerator": 0, "denominator": 1 });
    assert_eq!(
        rendered["worst_absolute_deviation"], zero,
        "the sole widest window is the very centre deviations are measured from, so its worst \
         absolute deviation is exactly zero — the identity restated through the report's own \
         arithmetic rather than only by value"
    );
    assert_eq!(
        rendered["spread"], zero,
        "one window has no other window to spread against"
    );
}

/// An admitted replicate carrying the strictly increasing fixture series.
fn increasing_replicate(samples: &[u128]) -> CompleteReplicate {
    let line = LedgerFixture::complete(0)
        .with_samples(samples.to_vec())
        .line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("the fixture line decodes")
        .into_lines();
    CompleteReplicate::admit(&lines[0].record).expect("the fixture line is admissible")
}
