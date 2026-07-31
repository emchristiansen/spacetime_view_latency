//! A campaign in which no attempt completed selects no attempt.

use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;

use super::fixture;

/// Coverage: the only path through the selection fold a pure test can reach, together with the
/// reason it is the only one — stated here rather than left as a silent gap.
///
/// What it reaches is not nothing. The fold walks all sixty frozen slots, takes the no-candidate arm
/// of its outcome match at every attempt, and returns an empty result rather than panicking or
/// inventing a selection. A fold that read "lowest ordinal for this slot" without first filtering on
/// the outcome would return sixty selections here, each over an attempt that measured nothing; one
/// that crossed the provisioning bridge before checking the outcome would panic on the first
/// unprovisioned record.
///
/// The first assertion pins that premise rather than trusting it. If the baseline fixture ever
/// gained a complete attempt, this test would otherwise keep passing while testing something else
/// entirely.
///
/// **What it cannot reach**, all of it downstream of a candidate existing at all: minting a
/// selection, carrying the admitted provenance onto it, excluding a superseded record *before* the
/// ordinal minimum, and competing two ordinals. Each needs a
/// [`Complete`](AttemptOutcome::Complete) attempt inside an *accepted* ledger, and reconciliation
/// requires a provisioned line for every complete attempt, whose
/// [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance) only
/// a live distribution, running server, and published module can mint. No ledger built in this
/// process can carry one, so those behaviors remain compile-checked and directly inspected rather
/// than closed off with a forgeable test-only provenance seam.
///
/// Lowest-ordinal *competition* is unreachable for a second and independent reason, which would
/// survive even a provisioning-capable fixture: reconciliation admits a retry only when its
/// original's
/// [`retry_eligibility`](crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord::retry_eligibility)
/// is `Eligible`, and a complete original never is — so at most one complete attempt per slot is
/// representable under today's retry policy. The general minimum the spec states is implemented as
/// stated, rather than narrowed into an assertion about that policy, because the policy is the thing
/// that could change.
#[test]
fn a_campaign_with_no_complete_attempt_selects_nothing() {
    let campaign = fixture::full_campaign().accepted();

    assert!(
        campaign
            .attempts()
            .iter()
            .all(|record| !matches!(record.terminal().outcome(), AttemptOutcome::Complete { .. })),
        "the baseline this asserts over must genuinely contain no complete attempt"
    );
    assert!(
        campaign.selections().is_empty(),
        "a campaign in which no attempt completed can select no attempt"
    );
}
