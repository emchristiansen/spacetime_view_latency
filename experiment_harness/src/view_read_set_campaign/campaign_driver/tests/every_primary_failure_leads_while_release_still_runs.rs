//! Each of the four ways settling can fail leads the returned error, and none of them stops release
//! from running or hides what release reported.

use anyhow::{anyhow, Error};

use crate::view_read_set_campaign::campaign_driver::settle;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::capturing_writer::CapturingWriter;
use super::fixture;
use super::scripted_writer::ScriptedWriter;

/// Coverage: the four primary sources — the outcome arriving already failed, the terminal append, the
/// post-attempt observation, and the post-attempt append. Each is asserted twice over: that it leads,
/// and that both release failures are still there behind it. A primary that masked release, or a
/// release that masked the primary, would pass one and fail the other.
///
/// Two cases assert what was *written* rather than what was reported, because "no line was attempted"
/// and "the line's error was dropped" are indistinguishable in an error chain.
///
/// Each case gets its own fresh sink, so nothing it reports is the previous case's poison.
#[test]
fn every_primary_failure_leads_while_release_still_runs() {
    let releasing_the_server = "releasing the server";
    let removing_the_data_directory = "removing the data directory";
    let release_pair = || {
        vec![
            anyhow!("{releasing_the_server}"),
            anyhow!("{removing_the_data_directory}"),
        ]
    };
    let assert_leads = |error: &Error, primary: &str| {
        let rendered = format!("{error:#}");
        let (Some(leads), Some(first_release), true) = (
            rendered.find(primary),
            rendered.find(releasing_the_server),
            rendered.contains(removing_the_data_directory),
        ) else {
            panic!("the primary and every release failure must all be visible: {rendered}")
        };
        assert!(
            leads < first_release,
            "the primary failure must lead, with release behind it: {rendered}"
        );
    };

    let record = fixture::record(RetryOrdinal::ORIGINAL, fixture::application_failure());
    let sample = fixture::post_attempt_sample();

    // The outcome was already a ledger persist failure: a further write would be dishonest, not
    // merely futile, so nothing is written at all.
    let earlier_persist_failure = "the persist failure that produced this outcome";
    let (writer, lines) = CapturingWriter::new();
    let mut sink = CampaignSink::from_writer(Box::new(writer));
    let error = settle(
        &mut sink,
        Err(anyhow!("{earlier_persist_failure}")),
        Some(Ok(sample)),
        release_pair,
    )
    .expect_err("an outcome that is already a persist failure cannot settle successfully");
    assert!(
        lines.borrow().is_empty(),
        "an already-failed outcome writes no ledger line"
    );
    assert_leads(&error, earlier_persist_failure);

    // The terminal append itself fails, which poisons the sink — so the post-attempt line is never
    // *attempted* on it. The call count is what proves that: an implementation that appended and
    // discarded the refusal would leave the error text identical.
    let (writer, writes) = ScriptedWriter::counted(0);
    let mut sink = CampaignSink::from_writer(Box::new(writer));
    let error = settle(
        &mut sink,
        Ok(record.clone()),
        Some(Ok(sample)),
        release_pair,
    )
    .expect_err("a failed terminal append cannot settle successfully");
    assert_leads(&error, "recording the attempt terminal outcome");
    assert_eq!(
        *writes.borrow(),
        1,
        "a poisoned sink must be handed the terminal line and nothing after it"
    );

    // The observation failed. The measured disposition is written unchanged — a retrospective
    // diagnostic may not decide it — but the campaign stops, because the ledger now permanently
    // lacks a line reconciliation requires.
    let failed_observation = "reading the post-attempt host environment";
    let (writer, lines) = CapturingWriter::new();
    let mut sink = CampaignSink::from_writer(Box::new(writer));
    let error = settle(
        &mut sink,
        Ok(record.clone()),
        Some(Err(anyhow!("{failed_observation}"))),
        release_pair,
    )
    .expect_err("a failed post-attempt observation stops the campaign");
    assert_eq!(
        lines.borrow().len(),
        1,
        "the terminal disposition is written whatever the observation did"
    );
    assert_leads(&error, failed_observation);

    // The terminal line lands and the post-attempt append is what fails.
    let mut sink = CampaignSink::from_writer(Box::new(ScriptedWriter::ok_for(1)));
    let error = settle(&mut sink, Ok(record), Some(Ok(sample)), release_pair)
        .expect_err("a failed post-attempt append stops the campaign");
    assert_leads(&error, "recording the post-attempt environment");
}
