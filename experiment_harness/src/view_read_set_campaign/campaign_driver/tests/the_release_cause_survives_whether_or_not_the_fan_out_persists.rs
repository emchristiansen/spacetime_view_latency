//! Stopping a campaign for an unproven release reports that release's cause on both paths, and puts
//! a ledger that stopped accepting lines in front of it.

use anyhow::anyhow;

use crate::view_read_set_campaign::campaign_driver::{release_failure, stop_unreleased};
use crate::view_read_set_campaign::campaign_sink::CampaignSink;

use super::capturing_writer::CapturingWriter;
use super::fixture;
use super::scripted_writer::ScriptedWriter;

/// Coverage: the release cause is the only account of why the campaign stopped, so no second
/// failure may consume it. Both paths run from one release failure, since what must not vary is
/// what they have in common.
///
/// Order is the assertion with teeth: `into_error` enumerates, so a reversed aggregation would
/// contain both strings and still report a stopped ledger as a consequence of a teardown. The
/// failing path also proves the message drops the fan-out's claim — nothing was recorded, and a
/// test searching only for the release text would pass while the error asserted a fan-out that
/// never happened.
#[test]
fn the_release_cause_survives_whether_or_not_the_fan_out_persists() {
    let releasing_the_server = "releasing the server";
    let inventory = fixture::inventory();
    let attempts = inventory.attempts();
    let position = 2;
    assert!(
        attempts.len() > position + 1,
        "the frozen inventory must predeclare originals after position {position}"
    );
    let untouched = &attempts[position + 1..];
    let failing = attempts[position];

    let unreleased = || match release_failure(vec![anyhow!("{releasing_the_server}")]) {
        Some(unreleased) => unreleased,
        None => panic!("a non-empty release failure list is an unreleased-capabilities value"),
    };

    let (writer, _lines) = CapturingWriter::new();
    let mut healthy = CampaignSink::from_writer(Box::new(writer));
    let stopped = match stop_unreleased(&mut healthy, untouched, unreleased(), failing) {
        Ok(()) => panic!("an unproven release is campaign-terminal however the fan-out went"),
        Err(stopped) => format!("{stopped:#}"),
    };
    assert!(
        stopped.contains(releasing_the_server),
        "the release cause is why the campaign stopped and must reach its error: {stopped}"
    );
    assert!(
        stopped.contains("were recorded NotRun"),
        "a fan-out that persisted may say so: {stopped}"
    );

    // Refuses its very first line, so no skipped slot is recorded and nothing carries the cause.
    let mut terminal = CampaignSink::from_writer(Box::new(ScriptedWriter::ok_for(0)));
    let aggregated = match stop_unreleased(&mut terminal, untouched, unreleased(), failing) {
        Ok(()) => panic!("a ledger that refused the fan-out cannot report success"),
        Err(aggregated) => format!("{aggregated:#}"),
    };
    let ledger = match aggregated.find("scripted write_line failure") {
        Some(ledger) => ledger,
        None => panic!("the ledger's own refusal must reach the error: {aggregated}"),
    };
    let release = match aggregated.find(releasing_the_server) {
        Some(release) => release,
        None => panic!("the release cause must survive the fan-out's failure: {aggregated}"),
    };
    assert!(
        ledger < release,
        "a ledger that stopped accepting lines leads, with the release cause behind it: \
         {aggregated}"
    );
    assert!(
        !aggregated.contains("were recorded NotRun"),
        "no slot was recorded, so the error may not claim any were: {aggregated}"
    );
}
