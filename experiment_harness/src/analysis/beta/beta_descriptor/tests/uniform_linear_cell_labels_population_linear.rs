//! A cell whose 30 blocks are all the clean linear ladder `y = 2N` yields 30 identifiable `β = 1`
//! estimates, so the derived population interval exists, collapses to `[1, 1]`, and is labeled
//! linear-consistent (spec: only when all 30 block estimates are identifiable may the descriptor emit the
//! population interval and a linear/sublinear label).

use crate::analysis::beta::beta_descriptor::tests::{ladder_from, N_BLOCKS};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::beta::beta_label::BetaLabel;
use crate::analysis::beta::block_point::BlockPoint;
use crate::params::NUM_DOSES_USIZE;

#[test]
fn uniform_linear_cell_labels_population_linear() {
    // 30 identical linear ladders → 30 identifiable β = 1 fits.
    let block = ladder_from(|n| 2.0 * n);
    let ladders: [[BlockPoint; NUM_DOSES_USIZE]; N_BLOCKS] = [block; N_BLOCKS];
    let descriptor = BetaDescriptor::from_ladders(&ladders);

    assert!(
        descriptor.blocks().iter().all(|fit| fit.is_identifiable()),
        "every one of the 30 clean linear blocks is identifiable"
    );

    let population = descriptor
        .population()
        .expect("an all-identifiable cell emits the population interval");
    // All 30 exponents are 1, so both order statistics are 1: a degenerate [1, 1] interval.
    assert!(
        (population.interval_lo() - 1.0).abs() < 1e-6,
        "the population lower order statistic is β = 1, got {}",
        population.interval_lo()
    );
    assert!(
        (population.interval_hi() - 1.0).abs() < 1e-6,
        "the population upper order statistic is β = 1, got {}",
        population.interval_hi()
    );
    assert!(
        matches!(population.label(), BetaLabel::LinearConsistent),
        "an interval within [0.8, 1.2] is linear-consistent, got {:?}",
        population.label()
    );
    // The population reuses the primary classifier's frozen 95.7226% order-statistic coverage.
    assert!(
        (population.coverage() - 0.957226).abs() < 1e-5,
        "the population interval reports the frozen 95.7226% coverage, got {}",
        population.coverage()
    );
}
