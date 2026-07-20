//! The population selector sorts its 30 identified exponents and takes the frozen `[X_(10), X_(21)]`
//! (1-based) order statistics, independent of input order, and labels a fully-sublinear interval
//! sublinear-consistent (spec: the 30-fit `[X_(10), X_(21)]` interval and labels, reusing the primary
//! classifier's exact order indices and 95.7226% coverage).

use crate::analysis::beta::beta_descriptor::tests::N_BLOCKS;
use crate::analysis::beta::beta_label::BetaLabel;
use crate::analysis::beta::beta_population::BetaPopulation;

#[test]
fn population_interval_selects_order_statistics() {
    // 30 distinct exponents 0.50, 0.51, …, 0.79 supplied in descending order, so the proof also pins that
    // the selector sorts before indexing rather than trusting input order.
    let betas: [f64; N_BLOCKS] = std::array::from_fn(|index| 0.79 - 0.01 * index as f64);
    let population = BetaPopulation::from_identified(&betas);

    // Sorted ascending: X_(10) is the 10th value (0-based index 9) = 0.59, X_(21) the 21st = 0.70.
    assert!(
        (population.interval_lo() - 0.59).abs() < 1e-9,
        "the lower order statistic X_(10) is 0.59, got {}",
        population.interval_lo()
    );
    assert!(
        (population.interval_hi() - 0.70).abs() < 1e-9,
        "the upper order statistic X_(21) is 0.70, got {}",
        population.interval_hi()
    );
    // [0.59, 0.70] lies wholly within the open sublinear band (0.2, 0.8).
    assert!(
        matches!(population.label(), BetaLabel::SublinearConsistent),
        "an interval within (0.2, 0.8) is sublinear-consistent, got {:?}",
        population.label()
    );
    assert!(
        (population.coverage() - 0.957226).abs() < 1e-5,
        "the population interval reports the frozen 95.7226% coverage, got {}",
        population.coverage()
    );
}
