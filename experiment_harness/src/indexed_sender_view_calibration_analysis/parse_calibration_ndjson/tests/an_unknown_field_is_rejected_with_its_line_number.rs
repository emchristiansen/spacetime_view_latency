//! An unknown field fails the whole line, and the failure names which line.

use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;

/// Coverage: the closed wire contract, and position tagging across a multi-line file.
///
/// `deny_unknown_fields` is the only thing standing between this analyzer and evidence written under
/// a method it does not model — it is what turns "the pilot grew a field" from a silent
/// misinterpretation into a loud refusal. The line number matters for the same reason it does in the
/// campaign's ingest: an operator holding a two-thousand-line ledger needs to be told which line,
/// not merely that one of them was wrong.
#[test]
fn an_unknown_field_is_rejected_with_its_line_number() {
    let good = LedgerFixture::complete(0).line();
    let tampered = LedgerFixture::complete(1)
        .line()
        .replace("\"schedule_seed\":", "\"unexpected_new_method_fact\":");
    let ledger = format!("{good}\n{tampered}\n");

    let error = parse_calibration_ndjson(&ledger)
        .expect_err("an unrecognised field must fail the line rather than be ignored");
    assert_eq!(
        error.line_number(),
        2,
        "the second line is the tampered one, and the refusal says so"
    );
    assert!(
        error.diagnostic().contains("unexpected_new_method_fact"),
        "the diagnostic names the offending field so the disagreement is actionable; got {}",
        error.diagnostic()
    );

    parse_calibration_ndjson(&format!("{good}\n"))
        .expect("the untampered fixture decodes, so the refusal above is about the tampering");
}
