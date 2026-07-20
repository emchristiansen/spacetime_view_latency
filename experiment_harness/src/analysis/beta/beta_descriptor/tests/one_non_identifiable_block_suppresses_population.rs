//! A cell of 29 clean linear ladders plus one all-zero (non-positive-scale) block retains all 30 block
//! outcomes but derives no population interval — a single non-identifiable block suppresses it (spec:
//! "prove that one non-identifiable block suppresses the population β interval while retaining all block
//! outcomes").

use crate::analysis::beta::beta_descriptor::tests::{ladder_from, N_BLOCKS};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::block_point::BlockPoint;
use crate::params::NUM_DOSES_USIZE;

#[test]
fn one_non_identifiable_block_suppresses_population() {
    // 29 identifiable linear blocks and one all-zero block whose fit is NonPositiveScale.
    let linear = ladder_from(|n| 2.0 * n);
    let zero = ladder_from(|_n| 0.0);
    let mut ladders: [[BlockPoint; NUM_DOSES_USIZE]; N_BLOCKS] = [linear; N_BLOCKS];
    ladders[0] = zero;
    let descriptor = BetaDescriptor::from_ladders(&ladders);

    // Every block outcome is retained — the failure is reported, not dropped.
    assert_eq!(
        descriptor.block_search_outcomes().len(),
        N_BLOCKS,
        "all 30 block outcomes are retained"
    );
    assert!(
        matches!(descriptor.block_search_outcomes()[0].fit(), BlockFit::NonPositiveScale),
        "the all-zero block is retained as a NonPositiveScale failure, got {:?}",
        descriptor.block_search_outcomes()[0].fit()
    );
    let identifiable = descriptor
        .block_fits()
        .filter(|fit| fit.is_identifiable())
        .count();
    assert_eq!(
        identifiable,
        N_BLOCKS - 1,
        "exactly the 29 clean linear blocks are identifiable"
    );

    // The single non-identifiable block suppresses the population interval entirely.
    assert!(
        descriptor.population().is_none(),
        "one non-identifiable block suppresses the population β interval"
    );
}
