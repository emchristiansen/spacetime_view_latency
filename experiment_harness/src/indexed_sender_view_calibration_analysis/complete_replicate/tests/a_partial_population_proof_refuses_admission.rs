//! Every population count is checked, not only the append count.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::frozen_population::FrozenPopulation;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;

/// Coverage: the full composition predicate, one field at a time.
///
/// The pilot's `VerifiedPopulation` is a capability token minted only by the row-by-row verifier —
/// and that token does not survive serialization. What reaches the ledger is three plain integers.
/// Checking only `verified_appends` would therefore admit a record whose arm or witness cache was
/// proven at some other size, re-opening the same-cardinality-substitution class the token exists to
/// exclude at exactly the boundary where the token is gone.
///
/// Each count is perturbed alone, so a check that happened to cover two fields at once would still
/// leave one case failing.
#[test]
fn a_partial_population_proof_refuses_admission() {
    let frozen = FrozenPopulation::complete();

    assert_eq!(
        refusal(&LedgerFixture::complete(0).with_arm_rows(frozen.arm_rows - 1).line()),
        (AdmissionRefusal::ArmRowsNotFrozen {
            arm_rows: frozen.arm_rows - 1,
            expected: frozen.arm_rows,
        }),
        "an arm cache proven one row short is not a complete attempt's proof"
    );

    assert_eq!(
        refusal(
            &LedgerFixture::complete(0)
                .with_witness_rows(frozen.witness_rows + 1)
                .line()
        ),
        (AdmissionRefusal::WitnessRowsNotFrozen {
            witness_rows: frozen.witness_rows + 1,
            expected: frozen.witness_rows,
        }),
        "a witness cache proven one row over is not a complete attempt's proof"
    );

    assert_eq!(
        refusal(
            &LedgerFixture::complete(0)
                .with_verified_appends(frozen.verified_appends - 1)
                .line()
        ),
        (AdmissionRefusal::VerifiedAppendsNotFrozen {
            verified_appends: frozen.verified_appends - 1,
            expected: frozen.verified_appends,
        }),
        "a series whose composition was verified against fewer appends than it has samples cannot \
         be a complete replicate"
    );
}

/// Decode one line, attempt admission, and return the refusal it must produce.
fn refusal(line: &str) -> AdmissionRefusal {
    let records = parse_calibration_ndjson(line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0])
        .expect_err("a perturbed population proof must refuse admission")
}
