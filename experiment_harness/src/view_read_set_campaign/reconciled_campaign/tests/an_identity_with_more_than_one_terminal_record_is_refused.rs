//! An identity the ledger terminated twice is refused.

use crate::view_read_set_campaign::campaign_record::CampaignRecord;

use super::fixture;

/// Coverage: "exactly one terminal disposition per attempt" is the assumption every later rule
/// rests on. The per-attempt clearance, provisioning and post-attempt rules are all stated *total
/// over the terminal outcome*, so an identity with two outcomes has two different sets of required
/// auxiliary lines, and whichever the reconciler read first would decide. Selection is worse: two
/// terminal records for one identity could disagree about whether the attempt completed at all.
///
/// The duplicate here is a genuinely *conflicting* disposition — the slot was already
/// preflight-rejected, and the second line says it launched and failed — so the ledger asserts both
/// that the attempt never ran and that it did. It is appended bare, with none of the auxiliary
/// lines that second outcome would require, and the refusal is still the duplicate-terminal one:
/// identity accounting comes before the per-attempt rules, which is the right order, since those
/// rules cannot be applied at all until it is settled which outcome an identity has.
///
/// The refusal names both sequences, because knowing an identity was terminated twice is only
/// actionable together with where.
#[test]
fn an_identity_with_more_than_one_terminal_record_is_refused() {
    let duplicated = fixture::eligible_slot();
    let mut ledger = fixture::full_campaign();
    let second = ledger.append(CampaignRecord::Terminal {
        record: fixture::terminal(duplicated, fixture::failed_before_publish(duplicated)),
    });

    let rendered = ledger.refused("an identity terminated twice must be refused");
    assert!(
        rendered.contains(&duplicated.canonical_tag()),
        "the refusal must name the identity terminated twice, got {rendered}"
    );
    assert!(
        rendered.contains(&second.get().to_string()),
        "the refusal must name the sequence of the duplicate terminal line, got {rendered}"
    );
}
