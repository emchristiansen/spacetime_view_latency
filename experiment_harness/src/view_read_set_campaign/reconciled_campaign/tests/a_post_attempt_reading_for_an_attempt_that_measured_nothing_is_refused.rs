//! A post-attempt environment reading for an attempt that measured nothing is refused.

use crate::view_read_set_campaign::campaign_record::CampaignRecord;

use super::fixture;

/// Coverage: the reachable direction of the post-attempt rule. The reading is taken *immediately
/// after a measured attempt*, so one filed against an attempt the gate refused, or one that failed
/// before publishing, describes a host state no measurement of this campaign was ever taken in.
/// Accepting it would put a reading into the ledger that a later reader could pair with evidence it
/// has nothing to do with.
///
/// Both cases are attempts that measured nothing for *different* reasons — one never launched, one
/// launched and failed before publish — because the rule is total over the outcome rather than a
/// single "did it measure" flag, and a reconciler could get one arm right and the other wrong.
///
/// **What this cannot reach**, and why it is stated here rather than left as an untested gap: the
/// other direction, an attempt that *did* measure and is missing its required reading, and the
/// **adjacency** clause — that the reading sits at exactly the sequence after its terminal line,
/// with nothing in between, which is what the spec's "immediately after" means. It is enforced by a
/// private `RequiredPosition::ImmediatelyAfterTerminal` inside the reconciler's sealed module, which
/// this sibling cannot name; the boundary that applies it is
/// [`ReconciledCampaign::reconciled`](crate::view_read_set_campaign::reconciled_campaign::ReconciledCampaign::reconciled).
/// Both need an outcome requiring exactly one reading, which means a measured attempt,
/// which means a published instance and therefore an
/// [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance) that
/// only a live server can mint. For every outcome constructible here the required count is zero, so
/// cardinality refuses first and the adjacency clause is never evaluated. Both remain
/// compile-checked and directly inspected rather than closed with a fabricated provenance.
///
/// The clearance family is *not* a substitute for that coverage, and deliberately so: its frozen
/// clause requires only that a clearance precede its terminal line, not that it be adjacent to it,
/// so the position rule it exercises is the weaker of the two.
#[test]
fn a_post_attempt_reading_for_an_attempt_that_measured_nothing_is_refused() {
    for (description, attempt) in [
        (
            "an attempt the preflight gate refused",
            fixture::eligible_slot(),
        ),
        (
            "an attempt that failed before publishing",
            fixture::failed_slot(),
        ),
    ] {
        let mut ledger = fixture::full_campaign();
        ledger.append(CampaignRecord::PostAttemptEnvironment {
            attempt,
            sample: fixture::post_attempt_sample(),
        });

        let rendered = ledger.refused(&format!(
            "a post-attempt reading for {description} must be refused"
        ));
        assert!(
            rendered.contains(&attempt.canonical_tag()),
            "the refusal for {description} must name the attempt, got {rendered}"
        );
        assert!(
            rendered.contains("has 1 post-attempt environment line(s), but its terminal outcome \
                               requires exactly 0"),
            "the refusal for {description} must report the post-attempt cardinality, got {rendered}"
        );
    }
}
