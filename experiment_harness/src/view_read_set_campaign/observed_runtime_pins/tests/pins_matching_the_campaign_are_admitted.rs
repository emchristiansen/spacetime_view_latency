//! An attempt whose five facts equal the campaign's pins is admitted.

use super::fixture;

/// Coverage: every other test in this tree asserts a *refusal*, and a gate that rejected everything
/// would pass all five of them. This is the one that says the gate admits the case a correct
/// campaign produces — an attempt run on the pinned distribution, publishing the pinned module,
/// under the campaign's confirmed-read setting.
///
/// It matters more here than a passing case usually does, because the gate is all-or-nothing: a
/// disagreement fails the *whole* reconciliation rather than excluding one attempt, so a
/// false refusal would discard an entire campaign's evidence.
///
/// What this establishes is the comparison's wiring — each field checked against the pin it names.
/// It does not establish that the frozen constants are the externally right values to pin; nothing
/// in this process can, since both sides read the same constants.
#[test]
fn pins_matching_the_campaign_are_admitted() {
    let pinned = fixture::pinned_version();

    fixture::agreeing(&pinned)
        .agrees_with(&fixture::campaign())
        .expect("observations equal to the campaign's own pins must be admitted");
}
