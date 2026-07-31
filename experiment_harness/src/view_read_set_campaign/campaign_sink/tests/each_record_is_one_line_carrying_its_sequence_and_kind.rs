//! Every appended record becomes exactly one line, at the next sequence, carrying exactly that
//! record's own serialization under its kind tag.

use crate::view_read_set_campaign::campaign_sink::CampaignSink;

use super::capturing_writer::CapturingWriter;
use super::fixture;

/// Coverage: the facts a ledger reader needs before it can enforce any of the ordering and
/// accounting invariants reconciliation checks — asserted over the real serialization of real
/// records, not assumed:
///
/// - **one line per record**, so a write is never split, coalesced, or able to forge a second line
///   from one body;
/// - **contiguous sequences from zero**, so "first line" and "before" are readable from the file
///   rather than from write-time knowledge — and so the sequence advances exactly once per
///   successful write, which is the observable half of "advance only after full durability";
/// - **exactly `{seq, body}`** — those two top-level keys and no others — with a body *structurally
///   equal* to the JSON value the record serializes to on its own. The expected value is computed
///   from each record *before* it is handed over, so this is an equality against the record rather
///   than a spot check of a field or a tag. Structural, not bytewise: two renderings that differ in
///   object member order or in which characters they escape are the same value, and this asserts the
///   value.
///
/// The sequence is asserted twice over: once as the value [`CampaignSink::append`] *returned*, and
/// once as the value that reached the *line*. A sink that advanced a private counter correctly while
/// serializing a stale one would satisfy either alone.
///
/// [`CampaignRecord::Provisioned`](crate::view_read_set_campaign::campaign_record::CampaignRecord)
/// is deliberately absent: its provenance is snapshotted out of live provisioning capabilities,
/// which a unit test cannot own, so this covers five of the six shapes.
#[test]
fn each_record_is_one_line_carrying_its_sequence_and_kind() {
    let (writer, lines) = CapturingWriter::new();
    let mut sink = CampaignSink::from_writer(Box::new(writer));

    let mut expected = Vec::new();
    for (index, (record, kind)) in fixture::purely_constructible_records()
        .into_iter()
        .enumerate()
    {
        // Computed before the record moves into the sink: this is the value the line must carry.
        let body = serde_json::to_value(&record).expect("every campaign record serializes");
        let seq = sink
            .append(record)
            .expect("the capturing writer accepts the line");
        let position = u64::try_from(index).expect("a record index fits u64");
        assert_eq!(
            seq.get(),
            position,
            "record {index} must be assigned the next sequence"
        );
        expected.push((body, kind));
    }

    let lines = lines.borrow();
    assert_eq!(
        lines.len(),
        expected.len(),
        "each appended record must produce exactly one line"
    );

    for (index, (line, (body, kind))) in lines.iter().zip(&expected).enumerate() {
        assert_eq!(
            line.iter().filter(|byte| **byte == b'\n').count(),
            1,
            "line {index} must contain exactly one newline, so one record is one line"
        );
        assert_eq!(
            line.last(),
            Some(&b'\n'),
            "line {index} must be newline-terminated NDJSON"
        );

        let position = u64::try_from(index).expect("a line index fits u64");
        fixture::assert_ledger_line(index, line, position, body, kind);
    }
}
