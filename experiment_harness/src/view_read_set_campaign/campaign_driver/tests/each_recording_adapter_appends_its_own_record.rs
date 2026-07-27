//! Each recording adapter appends exactly its own record, and four calls fill the ledger's first
//! four lines in order.

use crate::view_read_set_campaign::campaign_driver::{
    record_inventory, record_post_attempt, record_preflight_cleared, record_terminal,
};
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::capturing_writer::CapturingWriter;
use super::fixture;

/// Coverage: the whole of what these four adapters do — each builds its own `CampaignRecord` variant
/// from exactly the payload it was handed, and appends it. The expected records are built here from
/// the same values the adapters are given, so this is an equality against the input rather than a
/// check that *some* plausible line appeared: an adapter that swapped two variants, or dropped a
/// field on the way into one, compiles and would pass any weaker assertion.
///
/// The four run against one sink so the lines are also proved contiguous from zero. That is what
/// makes discarding the returned sequence safe: consecutive appends occupy consecutive sequences, so
/// the positional rules reconciliation enforces are decided by *call order* alone.
///
/// What this cannot prove is that call order. Inventory-first, clearance-before-provisioning, and
/// post-attempt-immediately-after-terminal belong to `run_campaign`, `run_attempt`, and `settle`,
/// which are still `todo!()`; the order used here is only a convenient one.
#[test]
fn each_recording_adapter_appends_its_own_record() {
    let attempt = fixture::key(RetryOrdinal::ORIGINAL);
    let inventory = fixture::inventory();
    let provenance = fixture::campaign_provenance();
    let terminal = fixture::record(RetryOrdinal::ORIGINAL, fixture::preflight_rejected());
    let gate = fixture::passed_gate();
    let sample = fixture::post_attempt_sample();

    let expected = [
        CampaignRecord::Inventory {
            inventory: inventory.clone(),
            provenance: provenance.clone(),
        },
        CampaignRecord::PreflightCleared { attempt, gate },
        CampaignRecord::Terminal {
            record: terminal.clone(),
        },
        CampaignRecord::PostAttemptEnvironment { attempt, sample },
    ];

    let (writer, lines) = CapturingWriter::new();
    let mut sink = CampaignSink::from_writer(Box::new(writer));

    record_inventory(&mut sink, &inventory, &provenance).expect("the writer accepts the line");
    record_preflight_cleared(&mut sink, attempt, gate).expect("the writer accepts the line");
    record_terminal(&mut sink, &terminal).expect("the writer accepts the line");
    record_post_attempt(&mut sink, attempt, sample).expect("the writer accepts the line");

    let lines = lines.borrow();
    assert_eq!(
        lines.len(),
        expected.len(),
        "each adapter must append exactly one line"
    );

    for (index, (line, record)) in lines.iter().zip(&expected).enumerate() {
        let parsed: serde_json::Value =
            serde_json::from_slice(line).expect("each ledger line must be one JSON object");
        let object = parsed
            .as_object()
            .expect("each ledger line must be a JSON object");

        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            ["body", "seq"],
            "line {index} must carry exactly the sequence and the body"
        );

        let position = u64::try_from(index).expect("a line index fits u64");
        assert_eq!(
            object["seq"].as_u64(),
            Some(position),
            "line {index} must sit at the next sequence, so four appends are contiguous from zero"
        );
        assert_eq!(
            object["body"],
            serde_json::to_value(record).expect("every campaign record serializes"),
            "line {index} must carry the record its adapter was handed, and no other"
        );
    }
}
