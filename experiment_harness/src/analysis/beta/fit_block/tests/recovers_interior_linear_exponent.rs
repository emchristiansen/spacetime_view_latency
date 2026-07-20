//! A purely linear ladder `y = 2N` recovers the interior exponent `β = 1` as an identifiable estimate
//! (spec: recover interior linear `β = 1`). The zero-intercept acceptance is proven separately by
//! [`accepts_exact_zero_intercept_fit`](super::accepts_exact_zero_intercept_fit); this proof owns only
//! the exponent-recovery obligation.

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

#[test]
fn recovers_interior_linear_exponent() {
    // y = 2N is exactly proportional to x = N^1, so the constrained fit is perfect at β = 1 (b ≈ 2,
    // RSS ≈ 0) and strictly worse at any other exponent — a distinguishable interior minimum.
    let fit = fit_block(&ladder_from(|n| 2.0 * n)).fit();

    let candidate = match fit {
        BlockFit::Identifiable(candidate) => candidate,
        other => panic!("a clean linear ladder must yield an identifiable interior fit, got {other:?}"),
    };
    assert!(
        (candidate.beta() - 1.0).abs() < 1e-6,
        "the recovered exponent is the interior linear β = 1, got {}",
        candidate.beta()
    );
    assert!(
        (candidate.b() - 2.0).abs() < 1e-6,
        "the recovered scale is the true slope b = 2, got {}",
        candidate.b()
    );
}
