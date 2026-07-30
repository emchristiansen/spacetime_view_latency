//! A period-two series is anti-correlated at odd lags and correlated at even ones.

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::lag_autocorrelation::LagAutocorrelation;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: that the diagnostic detects dependence *beyond* lag 1, and gets its sign right.
///
/// This is the test that would have caught freezing the domain at `k = 1`. A series alternating
/// between two values is perfectly anti-correlated at every odd lag and perfectly correlated at every
/// even one; a lag-1-only diagnostic would report strong negative dependence and say nothing about
/// the period-two structure that actually generated it — precisely the "and beyond" evidence §568's
/// effective-information wording depends on.
///
/// The signs are asserted on the **exact** numerator rather than on the normalized float, so the test
/// pins the arithmetic rather than a rendering.
#[test]
fn a_periodic_series_alternates_sign_by_lag() {
    let replicate = alternating_replicate();

    for lag in [1, 3, 5] {
        assert!(
            numerator_of(&replicate, lag) < Rational::zero(),
            "an alternating series is anti-correlated at odd lag {lag}"
        );
    }
    for lag in [2, 4, 6] {
        assert!(
            numerator_of(&replicate, lag) > Rational::zero(),
            "an alternating series is correlated at even lag {lag}, which a lag-1-only diagnostic \
             could not report at all"
        );
    }
}

/// The exact lag-`k` numerator, read back from the emitted projection.
///
/// Read from the rendered JSON rather than through an accessor, deliberately: what a reviewer will
/// actually hold is the artifact, so the test asserts on the same bytes rather than on an internal
/// this module could render differently.
fn numerator_of(replicate: &CompleteReplicate, lag: usize) -> Rational {
    let rendered = serde_json::to_value(LagAutocorrelation::of(replicate, lag))
        .expect("the diagnostic serializes");
    let numerator = &rendered["numerator"];
    Rational::new(
        i128::from(
            numerator["numerator"]
                .as_i64()
                .expect("the exact numerator renders as an integer"),
        ),
        i128::from(
            numerator["denominator"]
                .as_i64()
                .expect("the exact denominator renders as an integer"),
        ),
    )
}

/// A replicate alternating between two strictly positive values.
fn alternating_replicate() -> CompleteReplicate {
    let line = LedgerFixture::complete(0)
        .with_samples(
            (0..MAX_PACED_SAMPLES_USIZE)
                .map(|index| if index % 2 == 0 { 1_000_000 } else { 2_000_000 })
                .collect(),
        )
        .line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("the fixture line decodes")
        .into_lines();
    CompleteReplicate::admit(&lines[0].record)
        .expect("an alternating positive series is admissible")
}
