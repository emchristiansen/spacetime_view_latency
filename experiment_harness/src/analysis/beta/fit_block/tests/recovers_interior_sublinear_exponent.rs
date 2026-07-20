//! A square-root ladder `y = 100·√N` recovers the interior sublinear exponent `β = 0.5` as an
//! identifiable estimate (spec: recover interior sublinear `β = 0.5`).

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

#[test]
fn recovers_interior_sublinear_exponent() {
    // y = 100·N^0.5 is exactly proportional to x = N^0.5, so the constrained fit is perfect at β = 0.5
    // and strictly worse elsewhere — a distinguishable interior sublinear minimum.
    let fit = fit_block(&ladder_from(|n| 100.0 * n.sqrt())).fit();

    let candidate = match fit {
        BlockFit::Identifiable(candidate) => candidate,
        other => panic!("a clean square-root ladder must yield an identifiable interior fit, got {other:?}"),
    };
    assert!(
        (candidate.beta() - 0.5).abs() < 1e-6,
        "the recovered exponent is the interior sublinear β = 0.5, got {}",
        candidate.beta()
    );
    assert!(
        (candidate.b() - 100.0).abs() < 1e-4,
        "the recovered scale is the true coefficient b = 100, got {}",
        candidate.b()
    );
}
