//! Through the real file, the whole lifecycle: an exclusively created ledger holds exactly the lines
//! it was given, in order, and finalizes cleanly.

use crate::observation::output_path::OutputPath;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;

use super::fixture;
use super::scratch_dir::scratch_dir;

/// Coverage: the one test here that runs the production seam end to end — real exclusive creation,
/// real `write_line` with its flush and `sync_data`, real file-and-directory finalization — and then
/// reads the bytes back off disk. The fake writers elsewhere in this tree prove the sink's own
/// sequencing and poison logic precisely because they replace that seam; this proves the seam is
/// really the one the sink drives, and that a ledger on disk is what the sink believed it wrote.
///
/// **What "holds exactly the lines it appended" asserts.** Exactly as many lines as records, in the
/// order they were appended, each newline-terminated, each carrying the sequence its `append` call
/// returned under exactly the two top-level keys `seq` and `body`, with a body *structurally equal*
/// to the JSON value that record serializes to on its own. A ledger with an extra line, a missing
/// one, a reordered pair, an unterminated tail, or a body the sink truncated or enriched on the way
/// past would be readable and wrong. Structural, not bytewise: object member order and escaping are
/// not what this pins.
///
/// The clean `(unpoisoned, successful sync)` corner of finalization is asserted here rather than in
/// isolation, because a finalize that succeeded on an empty ledger would say less than one that
/// succeeds on a ledger with durable records in it.
#[test]
fn a_created_ledger_holds_exactly_the_lines_it_appended() {
    let dir = scratch_dir("holds-exactly-the-lines-it-appended");
    let path = dir.join("campaign.ndjson");
    let output = OutputPath::new(path.clone());

    let mut sink = CampaignSink::create(&output).expect("the exclusive create succeeds");

    let mut expected = Vec::new();
    for (index, (record, kind)) in fixture::purely_constructible_records()
        .into_iter()
        .enumerate()
    {
        // Computed before the record moves into the sink: this is the value the file must hold.
        let body = serde_json::to_value(&record).expect("every campaign record serializes");
        let seq = sink
            .append(record)
            .expect("the real writer persists the line");
        let position = u64::try_from(index).expect("a record index fits u64");
        assert_eq!(
            seq.get(),
            position,
            "record {index} must be assigned the next sequence"
        );
        expected.push((body, kind));
    }

    sink.finalize()
        .expect("a ledger with durable records and no poison finalizes cleanly");

    let contents = std::fs::read_to_string(&path).expect("the finalized ledger is readable");
    assert!(
        contents.ends_with('\n'),
        "the last record must be newline-terminated, like every other"
    );

    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(
        lines.len(),
        expected.len(),
        "the ledger must hold exactly one line per appended record"
    );

    for (index, (line, (body, kind))) in lines.iter().zip(&expected).enumerate() {
        let position = u64::try_from(index).expect("a line index fits u64");
        fixture::assert_ledger_line(index, line.as_bytes(), position, body, kind);
    }
}
