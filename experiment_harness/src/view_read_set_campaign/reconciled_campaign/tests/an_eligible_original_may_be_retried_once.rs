//! A retry of a slot whose original was preflight-rejected is accounted alongside that original.

use crate::view_read_set_campaign::attempt_inventory::CAMPAIGN_ATTEMPT_COUNT;

use super::fixture;

/// Coverage: the accepting half of the retry rule, and the reason the inventory rule cannot simply
/// be "every terminal identity is predeclared". The inventory is frozen before execution, so a
/// retry identity is in it nowhere — it is derived later, from an outcome that had not happened
/// yet. A reconciler that demanded predeclaration would reject every legitimate retry, and this is
/// the test that would catch it.
///
/// The original here was refused by the prospective gate, so nothing was measured and the retry
/// references no measured outcome — which is exactly the condition the rule turns on. The retry is
/// appended after the whole campaign rather than beside its original, because ledger adjacency is a
/// property of the driver's schedule and not something reconciliation requires: a retry accepted
/// here proves the accounting reads identity and eligibility rather than position.
#[test]
fn an_eligible_original_may_be_retried_once() {
    let mut ledger = fixture::full_campaign();
    ledger.append_attempt(fixture::eligible_retry(), fixture::preflight_rejected());

    let campaign = ledger.accepted();

    assert_eq!(
        campaign.attempts().len(),
        CAMPAIGN_ATTEMPT_COUNT + 1,
        "the ledger displays the retry alongside every predeclared original"
    );
    let retry = campaign
        .attempts()
        .last()
        .expect("a campaign accounting for sixty-one attempts has a last one");
    assert_eq!(
        retry.terminal().key(),
        fixture::eligible_retry(),
        "the retry must be accounted under its own identity, not folded into its original's"
    );
}
