//! The canonical ladder x-axis at its endpoints: dose 1 is exactly `BATCH_SIZE`, dose 10 is exactly
//! `NUM_DOSES * BATCH_SIZE`.

use crate::dataset::dose_index::DoseIndex;
use crate::params::{BATCH_SIZE, NUM_DOSES, NUM_DOSES_USIZE};

#[test]
fn cumulative_driving_rows_at_the_endpoints() {
    let dose_one = DoseIndex::ALL[0];
    let dose_ten = DoseIndex::ALL[NUM_DOSES_USIZE - 1];

    assert_eq!(dose_one.get(), 1, "ALL starts at the first dose rung");
    assert_eq!(
        dose_one.cumulative_driving_rows(),
        BATCH_SIZE,
        "the first dose has driven exactly one batch"
    );

    assert_eq!(dose_ten.get(), NUM_DOSES, "ALL ends at the last dose rung");
    assert_eq!(
        dose_ten.cumulative_driving_rows(),
        NUM_DOSES * BATCH_SIZE,
        "the last dose has driven exactly NUM_DOSES batches"
    );
}
