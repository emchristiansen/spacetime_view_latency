//! Supersessions whose scopes reach attempts this ledger records are accounted and retained.

use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::method_supersession::MethodSupersession;
use crate::view_read_set_campaign::superseded_scope::SupersededScope;
use crate::view_read_set_campaign::supersession_justification::SupersessionJustification;

use super::fixture;

/// Coverage: the accepting side of the reach rule, in the one arrangement that actually pins it
/// down, plus both scope shapes.
///
/// **The discriminating case is the forward-only supersession.** The reach rule is deliberately not
/// strengthened to "reaches a terminal recorded *earlier* in the ledger", and only a scope that
/// reaches nothing earlier can tell the two rules apart. So one supersession here is scoped to the
/// retry identity and appended *before* that retry's terminal line: at the moment it is written,
/// every attempt it covers is still in the future. An earlier-only validator rejects it; the rule
/// as specified accepts it. The other two scopes cannot make that distinction — the
/// candidate-version scope already covers all sixty originals behind it, and the attempt scope on
/// the failed slot covers one — which is why they are exhaustive scope-shape coverage rather than
/// the discriminator.
///
/// A campaign really does append lines in this order: a finding about the measured code path is
/// meant to reach matching attempts written after it as well as before, and an attempt scope may
/// name an identity whose retry has been scheduled but not yet terminated.
#[test]
fn a_supersession_reaching_a_recorded_attempt_is_kept() {
    let mut ledger = fixture::full_campaign();
    ledger.append(CampaignRecord::Supersession {
        supersession: MethodSupersession::recorded(
            SupersededScope::Attempt(fixture::failed_slot()),
            SupersessionJustification::parsed("this attempt's seeding was later found unsound")
                .expect("a stated reason is nonempty after trimming"),
        ),
    });
    ledger.append(CampaignRecord::Supersession {
        supersession: MethodSupersession::recorded(
            SupersededScope::CandidateVersion {
                candidate: CandidateId::EntityOwnerSenderView,
                version: ENTITY_OWNER_SENDER_VIEW_VERSION,
            },
            SupersessionJustification::parsed("the measured view body was later found unsound")
                .expect("a stated reason is nonempty after trimming"),
        ),
    });
    ledger.append(CampaignRecord::Supersession {
        supersession: MethodSupersession::recorded(
            SupersededScope::Attempt(fixture::eligible_retry()),
            SupersessionJustification::parsed("the retry reused the unsound seeding")
                .expect("a stated reason is nonempty after trimming"),
        ),
    });
    ledger.append_attempt(fixture::eligible_retry(), fixture::preflight_rejected());

    let campaign = ledger.accepted();

    assert_eq!(
        campaign.supersessions().len(),
        3,
        "every recorded supersession reaches an attempt this ledger writes, so all are retained"
    );

    let backward_only = attempt_scoped(&campaign, fixture::failed_slot());
    let forward_only = attempt_scoped(&campaign, fixture::eligible_retry());
    let version_scoped = campaign
        .supersessions()
        .iter()
        .find(|supersession| {
            matches!(
                supersession.scope(),
                SupersededScope::CandidateVersion { .. }
            )
        })
        .expect("the candidate-version-scoped supersession is retained");

    assert!(
        forward_only.covers(fixture::eligible_retry())
            && !forward_only.covers(fixture::failed_slot()),
        "the discriminating supersession must reach only the retry, whose terminal line is written \
         after its own"
    );
    assert!(
        backward_only.covers(fixture::failed_slot())
            && !backward_only.covers(fixture::eligible_retry()),
        "an attempt scope names exactly its own identity: a retry is a different attempt with its \
         own evidence"
    );
    assert!(
        version_scoped.covers(fixture::failed_slot())
            && version_scoped.covers(fixture::eligible_retry()),
        "a candidate-version scope reaches every attempt of that candidate at that version, on \
         both sides of its own line"
    );
}

/// The retained supersession scoped to exactly `attempt`.
fn attempt_scoped<'a>(
    campaign: &'a crate::view_read_set_campaign::reconciled_campaign::ReconciledCampaign,
    attempt: AttemptKey,
) -> &'a MethodSupersession {
    campaign
        .supersessions()
        .iter()
        .find(|supersession| supersession.scope() == SupersededScope::Attempt(attempt))
        .expect("the supersession scoped to this attempt is retained")
}
