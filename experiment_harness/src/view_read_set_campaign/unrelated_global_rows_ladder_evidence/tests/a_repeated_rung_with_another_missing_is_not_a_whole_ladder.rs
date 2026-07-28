//! Six selections of one ladder that measure one rung twice and another never are not a whole
//! ladder.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::unrelated_global_rows_ladder_evidence::one_unrelated_global_rows_ladder;

use super::fixture;

/// Coverage: the rung multiset, the clause a count cannot stand in for — these six are the right
/// cardinality, all one ladder, and all distinct identities, so the only fault is that `S_last` was
/// never measured. The repeat is one slot's original and retry, the shape the selection fold could
/// plausibly emit and that comparing identities rather than rungs would admit.
#[test]
fn a_repeated_rung_with_another_missing_is_not_a_whole_ladder() {
    let [first, second, third, fourth, fifth, _sixth] = fixture::scales();
    let (block, _) = fixture::two_blocks();
    let role = RunRole::Arm;

    let repeated = [
        fixture::key(first, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(second, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(third, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(fourth, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(fifth, role, block, RetryOrdinal::ORIGINAL),
        fixture::key(first, role, block, RetryOrdinal::RETRY),
    ];

    let reported = match one_unrelated_global_rows_ladder(repeated) {
        Ok(()) => panic!("a ladder missing its last rung is not a whole ladder"),
        Err(reported) => format!("{reported:#}"),
    };
    assert!(
        reported.contains("exactly once"),
        "the rung multiset is what these six violate, and the error must say so: {reported}"
    );
}
