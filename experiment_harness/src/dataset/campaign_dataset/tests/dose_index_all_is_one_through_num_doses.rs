//! `DoseIndex::ALL` is exactly `1..=NUM_DOSES` in ascending order.

use crate::dataset::dose_index::DoseIndex;
use crate::params::NUM_DOSES;

/// The `const`-block-built `ALL` array is the sole source of `DoseIndex` values, so its contents
/// are the whole domain. Assert it has `NUM_DOSES` entries carrying the 1-based numbers
/// `1..=NUM_DOSES` in order — the runtime property the const block computes.
#[test]
fn dose_index_all_is_one_through_num_doses() {
    assert_eq!(
        DoseIndex::ALL.len(),
        NUM_DOSES as usize,
        "ALL has one entry per dose"
    );
    let values: Vec<u64> = DoseIndex::ALL.iter().map(|dose| dose.get()).collect();
    let expected: Vec<u64> = (1..=NUM_DOSES).collect();
    assert_eq!(
        values, expected,
        "ALL is exactly 1..=NUM_DOSES in ascending order"
    );
}
