//! A rung measured by the matched Control belongs to that role's ladder, not to the Arm's.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::unrelated_global_rows_ladder_evidence::one_unrelated_global_rows_ladder;

use super::fixture;

/// Coverage: the role clause, isolated — the rung multiset is complete, so the only fault is that
/// the last selection measured the direct-table control, whose ladder the Arm's is compared against
/// rather than pooled with. The error text is asserted to show which clause caught it.
#[test]
fn a_rung_under_the_matched_role_is_not_this_ladder() {
    let [first, second, third, fourth, fifth, sixth] = fixture::scales();
    let (block, _) = fixture::two_blocks();

    let mixed_roles = [
        fixture::key(first, RunRole::Arm, block, RetryOrdinal::ORIGINAL),
        fixture::key(second, RunRole::Arm, block, RetryOrdinal::ORIGINAL),
        fixture::key(third, RunRole::Arm, block, RetryOrdinal::ORIGINAL),
        fixture::key(fourth, RunRole::Arm, block, RetryOrdinal::ORIGINAL),
        fixture::key(fifth, RunRole::Arm, block, RetryOrdinal::ORIGINAL),
        fixture::key(sixth, RunRole::Control, block, RetryOrdinal::ORIGINAL),
    ];

    let reported = match one_unrelated_global_rows_ladder(mixed_roles) {
        Ok(()) => panic!("an Arm ladder may not take its last rung from the Control"),
        Err(reported) => format!("{reported:#}"),
    };
    assert!(
        reported.contains("different ladder"),
        "the shared-identity clause is what rejects a foreign role: {reported}"
    );
}
