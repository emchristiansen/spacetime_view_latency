//! A settled measured attempt writes its terminal line and then its post-attempt line, and both are
//! durable before any capability is released.

use std::rc::Rc;

use crate::view_read_set_campaign::campaign_driver::settle;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::capturing_writer::CapturingWriter;
use super::fixture;

/// Coverage: the adjacency reconciliation enforces as `terminal_seq.next()`, which no single append
/// can see and which the recording adapters' own tests therefore could not prove. Two consecutive
/// sequences with exactly the two expected bodies is that property.
///
/// `release` asserts from inside itself that both lines are already written: the closure form exists
/// precisely so a capability cannot be handed back before the disposition is durable, and a test that
/// only inspected the buffer afterwards would pass just as well if `release` ran first.
///
/// The pairing is an acknowledged approximation: every purely constructible outcome is a non-measured
/// one, so production would not attach a reading to this record. `settle` deliberately does not
/// inspect that correspondence — `run_attempt` owns it and reconciliation rejects it — so the
/// approximation is invisible to the behavior under test.
#[test]
fn a_settled_attempt_records_its_terminal_line_then_its_post_attempt_line() {
    let record = fixture::record(RetryOrdinal::ORIGINAL, fixture::application_failure());
    let sample = fixture::post_attempt_sample();

    let expected_record = serde_json::to_value(&record).expect("a terminal record serializes");
    let expected_lines = [
        serde_json::to_value(CampaignRecord::Terminal {
            record: record.clone(),
        })
        .expect("a terminal line serializes"),
        serde_json::to_value(CampaignRecord::PostAttemptEnvironment {
            attempt: record.key(),
            sample,
        })
        .expect("a post-attempt line serializes"),
    ];

    let (writer, lines) = CapturingWriter::new();
    let mut sink = CampaignSink::from_writer(Box::new(writer));

    let at_release = Rc::clone(&lines);
    let (settled, diagnostic) = settle(&mut sink, Ok(record), Some(Ok(sample)), || {
        assert_eq!(
            at_release.borrow().len(),
            expected_lines.len(),
            "release must run only after both ledger lines are durable"
        );
        Vec::new()
    })
    .expect("a written ledger and a clean release settle successfully");

    assert!(
        diagnostic.is_none(),
        "a release that failed at nothing yields no diagnostic"
    );
    assert_eq!(
        serde_json::to_value(&settled).expect("a terminal record serializes"),
        expected_record,
        "the record is returned unchanged, so its caller schedules a retry from the record on disk"
    );

    let lines = lines.borrow();
    assert_eq!(
        lines.len(),
        expected_lines.len(),
        "a measured attempt writes exactly its terminal and post-attempt lines"
    );

    for (index, (line, expected)) in lines.iter().zip(&expected_lines).enumerate() {
        let parsed: serde_json::Value =
            serde_json::from_slice(line).expect("each ledger line must be one JSON object");
        let position = u64::try_from(index).expect("a line index fits u64");

        assert_eq!(
            parsed["seq"].as_u64(),
            Some(position),
            "line {index} must sit at the next sequence, so the post-attempt line immediately \
             follows the terminal line"
        );
        assert_eq!(
            &parsed["body"], expected,
            "line {index} must carry the record settle was handed, and no other"
        );
    }
}
