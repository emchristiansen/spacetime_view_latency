//! A zero-variance series yields the typed undefined form, never a defaulted zero or a NaN.

use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::lag_autocorrelation::LagAutocorrelation;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::normalized_autocorrelation::NormalizedAutocorrelation;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: the `0/0` boundary, which is the one place this diagnostic could silently lie.
///
/// A constant series has zero energy on both sides of every lag, so the coefficient is genuinely
/// undefined. Reporting `0.0` would say "no lag dependence" — a claim about a series that cannot
/// support one — and a NaN is not valid authoritative JSON at all. The typed variant is the only
/// honest answer, and it is asserted at both ends of the domain so a guard that fired only for small
/// `k` would still fail here.
#[test]
fn a_constant_series_is_undefined_rather_than_zero() {
    let replicate = constant_replicate();

    for lag in [1, MAX_PACED_SAMPLES_USIZE - 1] {
        let computed = LagAutocorrelation::of(&replicate, lag);
        let rendered = serde_json::to_value(computed).expect("the diagnostic serializes");
        let normalized = &rendered["normalized"];

        assert_eq!(
            normalized,
            &serde_json::to_value(NormalizedAutocorrelation::UndefinedZeroVariance)
                .expect("the variant serializes"),
            "a constant series has no variance at lag {lag}, so the coefficient is the typed \
             undefined form rather than a defaulted zero"
        );
    }
}

/// A replicate whose samples are all the same strictly positive value.
fn constant_replicate() -> CompleteReplicate {
    let line = LedgerFixture::complete(0)
        .with_samples(vec![1_000_000; MAX_PACED_SAMPLES_USIZE])
        .line();
    let records = parse_calibration_ndjson(&line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0]).expect("a constant positive series is admissible")
}
