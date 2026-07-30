//! A key component the freeze cannot contain refuses admission rather than being ignored.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::frozen_candidate_version::frozen_candidate_version;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;

/// Coverage: the hole an earlier draft left open, closed from both of its sides.
///
/// That draft retained the single-valued key components as opaque JSON, reasoning that their source
/// enums have one variant each. The reasoning confused the *writer* with the *reader*: a field this
/// analyzer declines to deserialize is a field nothing ever checks, so any value at all would have
/// been admitted.
///
/// Two cases, chosen because they are the consequential ones. A **`Control` role** would let this
/// analyzer's per-`W` output be read as the Arm-versus-Control comparison §568 forbids the pilot to
/// produce. A **foreign candidate version** would let a series measured against different module
/// code join the pair as though it were a replicate of the same thing.
#[test]
fn a_forged_key_component_is_refused() {
    let control_role = LedgerFixture::complete(0).with_role("Control").line();
    assert_eq!(
        refusal(&control_role),
        AdmissionRefusal::KeyNotFrozen,
        "a Control line is refused by name: the freeze contains no Control attempt, and admitting \
         one would make the forbidden comparison available"
    );

    let foreign_version = LedgerFixture::complete(0)
        .with_version(frozen_candidate_version() + 1)
        .line();
    assert_eq!(
        refusal(&foreign_version),
        AdmissionRefusal::KeyNotFrozen,
        "a series from a different candidate version is not a replicate of this one"
    );

    let unmodified = LedgerFixture::complete(0).line();
    let records = parse_calibration_ndjson(&unmodified).expect("the fixture line decodes");
    assert!(
        CompleteReplicate::admit(&records[0]).is_ok(),
        "the unmodified fixture is admissible, so the refusals above are about the forged component \
         rather than about the fixture being wrong"
    );
}

/// Decode one line, attempt admission, and return the refusal it must produce.
fn refusal(line: &str) -> AdmissionRefusal {
    let records = parse_calibration_ndjson(line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0]).expect_err("a forged key component must refuse admission")
}
