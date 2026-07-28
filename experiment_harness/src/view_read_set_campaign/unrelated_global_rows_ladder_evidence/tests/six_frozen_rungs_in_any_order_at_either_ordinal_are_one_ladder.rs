//! The whole frozen rung set is a ladder however the six are ordered and whichever of them was
//! selected from a retry.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::unrelated_global_rows_ladder_evidence::one_unrelated_global_rows_ladder;

use super::fixture;

/// Coverage: the two things the rule must *not* read. Selections arrive in the inventory's seeded
/// per-block rotation, never ascending; and two rungs here are their slots' retries, each
/// legitimately that slot's own lowest survivor.
#[test]
fn six_frozen_rungs_in_any_order_at_either_ordinal_are_one_ladder() {
    let [first, second, third, fourth, fifth, sixth] = fixture::scales();
    let (block, _) = fixture::two_blocks();
    let role = RunRole::Arm;

    let shuffled = [
        fixture::key(fourth, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(first, role, block, RetryOrdinal::RETRY),
        fixture::key(sixth, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(second, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(fifth, role, block, RetryOrdinal::RETRY),
        fixture::key(third, role, block, RetryOrdinal::ORIGINAL),
    ];

    match one_unrelated_global_rows_ladder(shuffled) {
        Ok(()) => {}
        Err(error) => panic!("the whole frozen ladder is one ladder in any order: {error:#}"),
    }
}
