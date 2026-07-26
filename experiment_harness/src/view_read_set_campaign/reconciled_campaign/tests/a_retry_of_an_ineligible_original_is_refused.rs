//! A retry of a slot whose original the retry rule refuses to reopen is refused.

use super::fixture;

/// Coverage: the constraining half of the retry rule. A retry identity is not predeclared anywhere,
/// so nothing in the inventory can vouch for it — the only thing that authorizes one is the
/// disposition of the original it derives from, and this is the check that reads it.
///
/// The original here failed with an application fault, which is a *measured answer about the
/// candidate*: retrying it would be re-rolling an unfavourable result until a favourable one
/// appeared, and would put a measured outcome into the retry criterion, which the protocol forbids
/// outright. Without this check a driver bug — or a hand-appended line — could add a second attempt
/// at a slot that already answered, and selection would then have two complete attempts to choose
/// between where the protocol intended one.
///
/// The refusal names the eligibility the original actually had, rather than only that a retry was
/// unauthorized, since a reader's next question is which rule refused it.
#[test]
fn a_retry_of_an_ineligible_original_is_refused() {
    let mut ledger = fixture::full_campaign();
    ledger.append_attempt(fixture::ineligible_retry(), fixture::preflight_rejected());

    let rendered = ledger.refused("a retry of a slot whose original is ineligible must be refused");
    assert!(
        rendered.contains(&fixture::ineligible_retry().canonical_tag()),
        "the refusal must name the retry it refused, got {rendered}"
    );
    assert!(
        rendered.contains("Ineligible"),
        "the refusal must name the eligibility the original actually had, got {rendered}"
    );
}
