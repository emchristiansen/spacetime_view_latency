//! Exactly the frozen sample count is admitted; one short and one long are not.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: the count boundary, from both sides and at the exact value.
///
/// Both directions matter and for different reasons. A **short** series cannot answer the `W = 1000`
/// case at all, so admitting one would let the largest candidate be evaluated against evidence that
/// does not exist. A **long** series is evidence from a method other than the frozen one — `W_max`
/// bounds what an attempt may cost, so a series past it did not run the recipe the ledger claims.
#[test]
fn a_complete_replicate_is_exactly_the_frozen_series() {
    let exact = admit_with(MAX_PACED_SAMPLES_USIZE);
    let replicate = exact.expect("a series of exactly the frozen count is a replicate");
    assert_eq!(
        replicate.samples().len(),
        MAX_PACED_SAMPLES_USIZE,
        "admission preserves the whole series"
    );

    for short_or_long in [MAX_PACED_SAMPLES_USIZE - 1, MAX_PACED_SAMPLES_USIZE + 1] {
        assert_eq!(
            admit_with(short_or_long).unwrap_err(),
            AdmissionRefusal::SampleCountNotFrozen {
                samples: short_or_long
            },
            "{short_or_long} samples is not a replicate, and the refusal names the count it saw"
        );
    }
}

/// Admit a line whose series is `samples` long and strictly positive throughout.
fn admit_with(samples: usize) -> Result<CompleteReplicate, AdmissionRefusal> {
    let line = LedgerFixture::complete(0)
        .with_samples(
            (0..samples)
                .map(|index| 1_000_000 + index as u128)
                .collect(),
        )
        .line();
    let lines = CalibrationLedger::from_complete_contents_for_tests(&line)
        .expect("the fixture line decodes")
        .into_lines();
    CompleteReplicate::admit(&lines[0].record)
}
