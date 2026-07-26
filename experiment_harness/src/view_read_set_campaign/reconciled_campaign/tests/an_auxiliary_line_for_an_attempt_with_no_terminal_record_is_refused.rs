//! Auxiliary lines naming an identity the ledger never terminated are refused.

use crate::view_read_set_campaign::campaign_record::CampaignRecord;

use super::fixture;

/// Coverage: an orphan line is the case the per-attempt cardinality rules structurally cannot see.
/// Those rules are stated total over a terminal outcome — "this outcome requires exactly one
/// clearance" — so an identity with *no* terminal outcome has no rule to violate, and a reconciler
/// that only walked terminal records would silently ignore every line naming it.
///
/// What it means is a real failure: a clearance with no terminal record says an attempt was cleared
/// to launch and then vanished from the ledger without any disposition being written, which is
/// exactly the state the frozen inventory plus mandatory terminal records exists to make
/// impossible. A post-attempt reading with no terminal record says the same about a measured
/// attempt.
///
/// Both cases use a *retry* identity, which is the only orphan a full campaign ledger can have:
/// every original is predeclared and separately required to have a terminal record, so an
/// unterminated original is caught as a missing predeclared slot instead. A retry is predeclared
/// nowhere, so nothing else accounts for it.
#[test]
fn an_auxiliary_line_for_an_attempt_with_no_terminal_record_is_refused() {
    let orphaned = fixture::eligible_retry();

    let mut orphan_clearance = fixture::full_campaign();
    orphan_clearance.append(CampaignRecord::PreflightCleared {
        attempt: orphaned,
        gate: fixture::passed_gate(),
    });

    let mut orphan_post_attempt = fixture::full_campaign();
    orphan_post_attempt.append(CampaignRecord::PostAttemptEnvironment {
        attempt: orphaned,
        sample: fixture::post_attempt_sample(),
    });

    for (description, ledger) in [
        (
            "a clearance for an identity that never terminated",
            orphan_clearance,
        ),
        (
            "a post-attempt reading for an identity that never terminated",
            orphan_post_attempt,
        ),
    ] {
        let rendered = ledger.refused(&format!("{description} must be refused"));
        assert!(
            rendered.contains(&orphaned.canonical_tag()),
            "{description} must be refused by naming the orphaned identity, got {rendered}"
        );
        assert!(
            rendered.contains("no terminal record of its own"),
            "{description} must be refused as an orphan rather than as a cardinality error, got \
             {rendered}"
        );
    }
}
