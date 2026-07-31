//! A ledger that never accounts for a slot the frozen inventory predeclares is refused.

use super::fixture;

/// Coverage: the completeness half of the identity model. The inventory is written before execution
/// precisely so that a slot which never produced a terminal record is *detectable* — an attempt
/// that was never reached is still a known member, and the protocol's answer for it is a `NotRun`
/// record, not silence. Without this check a campaign that died partway through would reconcile
/// cleanly as a smaller campaign, and analysis would compute over a ladder missing rungs it never
/// learned were missing.
///
/// The second case distinguishes a bare gap from a gap with a retry standing where the original
/// should be. That ledger violates two rules at once — the slot has no original, and a retry has no
/// eligible original to derive from — and both refusals are correct. The inventory rule is checked
/// first deliberately, because naming the unaccounted slot is more actionable than naming the retry
/// that depended on it; this test pins that reported order down rather than leaving it to chance.
#[test]
fn a_predeclared_slot_with_no_terminal_record_is_refused() {
    let omitted = fixture::eligible_slot();

    let bare_gap = fixture::campaign_omitting(Some(omitted));
    let mut gap_with_retry = fixture::campaign_omitting(Some(omitted));
    gap_with_retry.append_attempt(fixture::eligible_retry(), fixture::preflight_rejected());

    for (description, ledger) in [
        ("a ledger missing a predeclared slot", bare_gap),
        (
            "a ledger whose missing slot has a retry but no original",
            gap_with_retry,
        ),
    ] {
        let rendered = ledger.refused(&format!("{description} must be refused"));
        assert!(
            rendered.contains(&omitted.canonical_tag()),
            "{description} must be refused by naming the unaccounted slot, got {rendered}"
        );
        assert!(
            rendered.contains("no terminal record"),
            "{description} must be refused for the missing terminal record, got {rendered}"
        );
    }
}
