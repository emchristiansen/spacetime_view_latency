//! An attempt that measured nothing writes only its terminal line, and its release failures come
//! back as the returned diagnostic rather than as an error.

use anyhow::anyhow;

use crate::view_read_set_campaign::campaign_driver::settle;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::capturing_writer::CapturingWriter;
use super::fixture;

/// Coverage: the `None` path and the healthy-ledger release path, which are the same run. A preflight
/// rejection is exactly the production shape here — nothing was measured, so no post-attempt reading
/// exists and reconciliation requires none.
///
/// With the ledger healthy, release failures are a *disposition*, not the campaign's error: the
/// caller records them against every remaining slot. Both are asserted present, because aggregating
/// is what stops a teardown failure hiding behind a disconnect failure.
#[test]
fn a_measureless_attempt_records_no_post_line_and_reports_its_release_failures() {
    let releasing_the_server = "releasing the server";
    let removing_the_data_directory = "removing the data directory";

    let record = fixture::record(RetryOrdinal::ORIGINAL, fixture::preflight_rejected());
    let expected_record = serde_json::to_value(&record).expect("a terminal record serializes");
    let expected_line = serde_json::to_value(CampaignRecord::Terminal {
        record: record.clone(),
    })
    .expect("a terminal line serializes");

    let (writer, lines) = CapturingWriter::new();
    let mut sink = CampaignSink::from_writer(Box::new(writer));

    let (settled, diagnostic) = settle(&mut sink, Ok(record), None, || {
        vec![
            anyhow!("{releasing_the_server}"),
            anyhow!("{removing_the_data_directory}"),
        ]
    })
    .expect("a written ledger settles successfully however the release went");

    assert_eq!(
        serde_json::to_value(&settled).expect("a terminal record serializes"),
        expected_record,
        "the record is returned unchanged even when release failed"
    );

    let rendered = format!(
        "{:?}",
        diagnostic.expect("release failures become the returned diagnostic")
    );
    assert!(
        rendered.contains(releasing_the_server) && rendered.contains(removing_the_data_directory),
        "every release failure must survive into the diagnostic: {rendered}"
    );

    let lines = lines.borrow();
    assert_eq!(
        lines.len(),
        1,
        "an attempt that measured nothing writes no post-attempt line"
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&lines[0]).expect("each ledger line must be one JSON object");
    assert_eq!(
        parsed["body"], expected_line,
        "the one written line is the terminal disposition"
    );
}
