//! A rung measured in another matched block belongs to that block's ladder, not to this one's.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::unrelated_global_rows_ladder_evidence::one_unrelated_global_rows_ladder;

use super::fixture;

/// Coverage: the block clause, isolated, and the likeliest violation in practice — the five blocks
/// share every other coordinate, so a fold grouping by candidate, axis, role and version alone would
/// assemble exactly this. Each block is an independent replicate the design keeps apart.
#[test]
fn a_rung_from_another_block_is_not_this_ladder() {
    let [first, second, third, fourth, fifth, sixth] = fixture::scales();
    let (block, other_block) = fixture::two_blocks();
    let role = RunRole::Arm;

    let mixed_blocks = [
        fixture::key(first, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(second, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(third, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(fourth, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(fifth, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(sixth, role, other_block, RetryOrdinal::ORIGINAL),
    ];

    let reported = match one_unrelated_global_rows_ladder(mixed_blocks) {
        Ok(()) => panic!("a block's ladder may not take its last rung from another block"),
        Err(reported) => format!("{reported:#}"),
    };
    assert!(
        reported.contains("different ladder"),
        "the shared-identity clause is what rejects a foreign block: {reported}"
    );
}
