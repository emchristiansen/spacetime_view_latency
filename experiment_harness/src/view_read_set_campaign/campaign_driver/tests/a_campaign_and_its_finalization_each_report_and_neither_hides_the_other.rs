//! Every combination of a finished campaign and a finalized ledger reports exactly what failed.

use anyhow::anyhow;

use crate::view_read_set_campaign::campaign_driver::campaign_outcome;

/// Coverage: all four corners, and the one that matters is both failing — a campaign that stopped
/// and a ledger that may not be durable are separate facts, and reporting the worse of the two would
/// send an operator to investigate half of what is unknown.
///
/// Order is asserted by byte offset, not presence: the body leads because finalization is what
/// happened afterward, and a reversed aggregate would contain both strings.
#[test]
fn a_campaign_and_its_finalization_each_report_and_neither_hides_the_other() {
    let body_failed = "the campaign stopped";
    let sync_failed = "the final sync failed";

    match campaign_outcome(Ok(()), Ok(())) {
        Ok(()) => {}
        Err(error) => panic!("a campaign that ran and a ledger that synced is success: {error:#}"),
    }

    for (walked, finalized, expected) in [
        (Err(anyhow!("{body_failed}")), Ok(()), body_failed),
        (Ok(()), Err(anyhow!("{sync_failed}")), sync_failed),
    ] {
        let reported = match campaign_outcome(walked, finalized) {
            Ok(()) => panic!("one failure is still a failed campaign"),
            Err(reported) => format!("{reported:#}"),
        };
        assert_eq!(
            reported, expected,
            "a lone failure is reported unchanged, with nothing wrapped around it"
        );
    }

    let both = match campaign_outcome(Err(anyhow!("{body_failed}")), Err(anyhow!("{sync_failed}")))
    {
        Ok(()) => panic!("two failures are not success"),
        Err(both) => format!("{both:#}"),
    };
    let body = match both.find(body_failed) {
        Some(body) => body,
        None => panic!("the campaign's own failure must survive finalization: {both}"),
    };
    let sync = match both.find(sync_failed) {
        Some(sync) => sync,
        None => panic!("a ledger that may not be durable must be reported too: {both}"),
    };
    assert!(
        body < sync,
        "the campaign's failure leads, with finalization's behind it: {both}"
    );
}
