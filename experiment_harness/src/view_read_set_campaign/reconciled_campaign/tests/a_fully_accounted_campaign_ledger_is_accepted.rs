//! A real sixty-slot campaign ledger, accounted exactly as the protocol requires, is accepted.

use crate::view_read_set_campaign::attempt_inventory::CAMPAIGN_ATTEMPT_COUNT;
use crate::view_read_set_campaign::reconciled_campaign::ProvisioningDisposition;

use super::fixture;

/// Coverage: the baseline every other test perturbs, and the only test that shows the accepting
/// path through all of it at once. A suite of refusals alone would be satisfied by a reconciler that
/// rejects everything, so what this establishes first is that a correct ledger is *not* rejected.
///
/// It then checks the three things acceptance is supposed to have produced, each of which a
/// reconciler could get wrong while still accepting:
///
/// - the inventory and pins come back as the ledger's own opening line stated them, since they are
///   taken from the stream rather than from the caller;
/// - every predeclared attempt is accounted, **in terminal-line order** — the order the ledger wrote
///   them, which for this fixture is the frozen execution order;
/// - every attempt's provisioning disposition is `NeverPublished`, which is the correct answer here
///   and is derived from each outcome's own typed lifecycle facts rather than from the absence of a
///   line. No outcome in this ledger published, so a reconciler that inferred "provisioned" from
///   anything else would disagree.
#[test]
fn a_fully_accounted_campaign_ledger_is_accepted() {
    let campaign = fixture::full_campaign().accepted();

    assert_eq!(
        campaign.inventory().attempts(),
        fixture::inventory().attempts(),
        "the accounted campaign must carry the inventory its own opening line froze"
    );
    assert_eq!(
        campaign.provenance().expected_release_commit(),
        fixture::provenance().expected_release_commit(),
        "the accounted campaign must carry the pins its own opening line recorded"
    );
    assert!(
        campaign.supersessions().is_empty(),
        "a ledger with no supersession lines must account for no supersessions"
    );

    assert_eq!(
        campaign.attempts().len(),
        CAMPAIGN_ATTEMPT_COUNT,
        "every predeclared attempt must be accounted for"
    );
    let accounted: Vec<_> = campaign
        .attempts()
        .iter()
        .map(|record| record.terminal().key())
        .collect();
    assert_eq!(
        accounted,
        fixture::inventory().attempts().to_vec(),
        "accounted attempts must be in the order the ledger wrote their terminal lines"
    );

    for record in campaign.attempts() {
        assert!(
            matches!(
                record.provisioning(),
                ProvisioningDisposition::NeverPublished
            ),
            "attempt {} never published, so it must be accounted as never provisioned",
            record.terminal().key().canonical_tag(),
        );
    }
}
