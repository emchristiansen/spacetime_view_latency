//! An attempt whose preflight could not be read is accounted with no auxiliary line at all.

use crate::view_read_set_campaign::attempt_inventory::CAMPAIGN_ATTEMPT_COUNT;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;

use super::fixture;

/// Coverage: the ledger contract that makes this a separate outcome rather than a `Failed` one. A
/// `Failed` attempt requires exactly one preflight clearance, and a clearance is minted only from a
/// *passing* gate — so an attempt whose readings never existed could not be recorded as failed
/// without producing a ledger this accounting refuses. Both directions are asserted: the campaign is
/// accepted with no clearance, no provisioned line and no post-attempt reading for that slot, and a
/// clearance added for it is refused.
///
/// The retry half of this outcome is not restated here. Reconciliation admits a retry by asking
/// [`retry_eligibility`](crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord::retry_eligibility),
/// which is proved over this outcome beside that rule, and the admission path itself is already
/// covered for the categorically-eligible class.
#[test]
fn an_unreadable_preflight_carries_no_auxiliary_lines() {
    let unreadable = fixture::spare_slot();
    let ledger = fixture::campaign_with_unreadable_preflight();

    let campaign = ledger.accepted();
    assert_eq!(
        campaign.attempts().len(),
        CAMPAIGN_ATTEMPT_COUNT,
        "every predeclared slot is still accounted for, including the unreadable one"
    );

    let mut cleared = fixture::campaign_with_unreadable_preflight();
    cleared.append(CampaignRecord::PreflightCleared {
        attempt: unreadable,
        gate: fixture::passed_gate(),
    });
    let rendered = cleared.refused("a clearance for an unreadable preflight must be refused");
    assert!(
        rendered.contains(&unreadable.canonical_tag()),
        "the refusal must name the attempt it accounts, got {rendered}"
    );
    assert!(
        rendered.contains("has 1 preflight clearance"),
        "the refusal must be the clearance accounting rather than some other rule, got {rendered}"
    );
}
