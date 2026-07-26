//! A supersession naming evidence this campaign never wrote is refused.

use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::method_supersession::MethodSupersession;
use crate::view_read_set_campaign::superseded_scope::SupersededScope;
use crate::view_read_set_campaign::supersession_justification::SupersessionJustification;

use super::fixture;

/// Coverage: an invalidation that reaches nothing is a deletion with nothing to dispute. It would
/// pass silently through a reconciler that merely collected supersession lines, and would then sit
/// in the ledger looking as though it had removed some evidence from consideration when it had not
/// — most likely because it names a mistyped or stale identity, and the evidence it *meant* to
/// invalidate is still being selected.
///
/// The scope used is a retry identity that is perfectly representable but that this ledger never
/// wrote a terminal record for, which is the realistic shape of the mistake: a retry that was
/// planned, or that a reader assumed had happened.
///
/// Only the attempt scope is exercised. A candidate-version scope cannot currently fail to reach:
/// every representable attempt is this one candidate at this one version, so any such scope reaches
/// all sixty. That becomes testable when a second candidate or version exists, and the rule is
/// written over `SupersededScope::covers` totally rather than per-variant so it will hold then
/// without change.
#[test]
fn a_supersession_reaching_no_recorded_attempt_is_refused() {
    let mut ledger = fixture::full_campaign();
    let unrecorded = fixture::eligible_retry();
    let seq = ledger.append(CampaignRecord::Supersession {
        supersession: MethodSupersession::recorded(
            SupersededScope::Attempt(unrecorded),
            SupersessionJustification::parsed("a finding against an attempt that was never made")
                .expect("a stated reason is nonempty after trimming"),
        ),
    });

    let rendered = ledger.refused(
        "a supersession reaching no attempt in this ledger must fail the whole reconciliation",
    );
    assert!(
        rendered.contains(&seq.get().to_string()),
        "the refusal must name the sequence of the supersession that reaches nothing, got {rendered}"
    );
    assert!(
        rendered.contains("reaches no attempt"),
        "the refusal must say the scope reaches no recorded attempt, got {rendered}"
    );
}
