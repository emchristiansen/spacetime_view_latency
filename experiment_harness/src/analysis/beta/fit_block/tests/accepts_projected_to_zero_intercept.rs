//! A power ladder with a negative offset, `y = 100·N^1.5 − 1000`, whose unconstrained fit has a negative
//! intercept is projected onto the `a = 0` edge and accepted as identifiable (spec: accept
//! projected-to-`a = 0` constrained solutions). This is the load-bearing counterpart to
//! [`accepts_exact_zero_intercept_fit`](super::accepts_exact_zero_intercept_fit): there the true
//! intercept is zero, here it is strictly negative and the `a ≥ 0` constraint is what forces `a = 0`.

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

#[test]
fn accepts_projected_to_zero_intercept() {
    // y = 100·N^1.5 − 1000 is exactly linear in x = N^1.5 with intercept −1000. At β = 1.5 the
    // unconstrained ordinary-least-squares intercept is exactly −1000 < 0, infeasible under a ≥ 0, so the
    // fit falls to the a = 0 edge b = Σxy/Σx². The offset is tiny against the ≈ 3.7·10^7 top of the
    // ladder, so the constrained optimum stays a distinguishable interior exponent near 1.5.
    let fit = fit_block(&ladder_from(|n| 100.0 * n.powf(1.5) - 1000.0)).fit();

    let candidate = match fit {
        BlockFit::Identifiable(candidate) => candidate,
        other => panic!("a negative-offset power ladder must yield an identifiable edge fit, got {other:?}"),
    };
    // The projection sets the intercept to the exact a = 0 edge — a literal zero, not a near-zero
    // unconstrained residue.
    assert_eq!(
        candidate.a(),
        0.0,
        "the negative unconstrained intercept is projected onto the exact a = 0 edge"
    );
    assert!(
        candidate.b() > 0.0,
        "the edge fit retains a strictly positive scale, got b = {}",
        candidate.b()
    );
}
