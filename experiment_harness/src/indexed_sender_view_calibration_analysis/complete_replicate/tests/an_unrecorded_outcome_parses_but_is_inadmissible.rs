//! Failed and never-run records decode cleanly and are refused by name, not by failing to parse.

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;

/// Coverage: the distinction between "this ledger is not the contract" and "this ledger is honest
/// but has no replicate here".
///
/// A run that recorded one failure and one never-reached slot produced a perfectly truthful
/// artifact. Refusing to *decode* it would report a malformed ledger for an honest one, and would
/// leave an operator unable to see which slots were lost or why. So both shapes parse, and both are
/// refused at admission with a reason naming what they actually are.
#[test]
fn an_unrecorded_outcome_parses_but_is_inadmissible() {
    let failed = LedgerFixture::complete(0).failed_line();
    let records = parse_calibration_ndjson(&failed).expect(
        "a failed attempt is a valid ledger line: refusing to decode it would call an honest \
         artifact malformed",
    );
    assert_eq!(
        CompleteReplicate::admit(&records[0]).unwrap_err(),
        AdmissionRefusal::AttemptFailed,
        "a failed attempt's retained samples are a rejected series, not a replicate"
    );

    let not_run = LedgerFixture::complete(1).not_run_line();
    let records = parse_calibration_ndjson(&not_run).expect("a never-run slot is a valid line");
    assert_eq!(
        CompleteReplicate::admit(&records[0]).unwrap_err(),
        AdmissionRefusal::NotAttempted,
        "a slot that never ran has no series to be a replicate of"
    );
}
