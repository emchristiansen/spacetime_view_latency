//! The `NotRun` fan-out records exactly the originals after the failing one, in frozen order, and
//! never the attempt that already has a terminal record.

use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
use crate::view_read_set_campaign::campaign_driver::record_not_run;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;
use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::not_run_reason::NotRunReason;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

use super::capturing_writer::CapturingWriter;
use super::fixture;

/// Coverage: the whole of what the fan-out decides — which identities are recorded, in which order,
/// at which sequences, and which are not. Expected bodies are built from the real frozen inventory
/// and the same diagnostic the helper is handed, so this is an equality against the input rather
/// than a check that plausible lines appeared.
///
/// **Exclusion is the off-by-one this exists to catch**: the failing attempt already wrote its own
/// terminal record, so a fan-out reaching one slot too far gives one identity two terminal lines —
/// which reconciliation refuses only after a whole campaign has run. Every earlier original is
/// checked too, since a wrong slice fails the same way further back. Each is asserted absent as the
/// exact body it would have carried, this helper writing one outcome and one only.
///
/// A middle position, so both sides are non-empty; which position does not enter the rule.
#[test]
fn the_skipped_slots_are_every_later_original_and_no_other() {
    let inventory = fixture::inventory();
    let attempts = inventory.attempts();
    let position = 2;
    assert!(
        attempts.len() > position + 1,
        "the frozen inventory must predeclare originals on both sides of position {position}"
    );
    let skipped = &attempts[position + 1..];
    let diagnostic = DiagnosticArtifact::of_error(&anyhow::anyhow!(
        "the attempt could not prove it released its server"
    ));

    let body = |attempt: AttemptKey| -> serde_json::Value {
        let outcome = AttemptOutcome::NotRun {
            reason: NotRunReason::PriorAttemptReleaseFailed {
                diagnostic: diagnostic.clone(),
            },
        };
        let record = match TerminalAttemptRecord::sealed(attempt, outcome) {
            Ok(record) => record,
            Err(error) => panic!(
                "a NotRun outcome carries no evidence to disagree with its identity: {error:#}"
            ),
        };
        match serde_json::to_value(CampaignRecord::Terminal { record }) {
            Ok(body) => body,
            Err(error) => panic!("every campaign record serializes: {error}"),
        }
    };

    let (writer, lines) = CapturingWriter::new();
    let mut sink = CampaignSink::from_writer(Box::new(writer));
    match record_not_run(&mut sink, skipped, &diagnostic) {
        Ok(()) => {}
        Err(error) => panic!("a healthy ledger accepts every skipped slot: {error:#}"),
    }

    let lines = lines.borrow();
    let written: Vec<serde_json::Value> = lines
        .iter()
        .map(|line| match serde_json::from_slice(line) {
            Ok(parsed) => parsed,
            Err(error) => panic!("each ledger line must be one JSON object: {error}"),
        })
        .collect();

    assert_eq!(
        written.len(),
        skipped.len(),
        "one line per skipped slot, and no line for any other"
    );
    for (index, (line, attempt)) in written.iter().zip(skipped).enumerate() {
        let sequence = match u64::try_from(index) {
            Ok(sequence) => sequence,
            Err(error) => panic!("a line index fits u64: {error}"),
        };
        assert_eq!(
            line["seq"].as_u64(),
            Some(sequence),
            "line {index} must sit at the next sequence, so the fan-out is contiguous from zero"
        );
        assert_eq!(
            line["body"],
            body(*attempt),
            "line {index} must record the skipped slot at that position of the frozen order, \
             carrying the release failure that skipped it"
        );
    }

    for excluded in &attempts[..=position] {
        let included = body(*excluded);
        assert!(
            !written.iter().any(|line| line["body"] == included),
            "the failing attempt and every original before it already terminated, so none may be \
             recorded NotRun: {} was",
            excluded.canonical_tag()
        );
    }
}
