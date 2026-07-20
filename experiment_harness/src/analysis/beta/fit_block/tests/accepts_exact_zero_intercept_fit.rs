//! A pure power ladder generated with an exact zero intercept, `y = 3N²`, is fit with an *exactly* zero
//! intercept from the unconstrained solution itself — the natural `a = 0` is accepted, not replaced by a
//! spurious positive intercept and not reached via the `a = 0` projection edge (spec: accept exact `a = 0`
//! constrained solutions). This is the load-bearing counterpart to
//! [`accepts_projected_to_zero_intercept`](super::accepts_projected_to_zero_intercept): there the true
//! intercept is strictly negative and the `a ≥ 0` constraint *forces* the edge; here the unconstrained
//! optimum is already `a = 0`, feasible, and retained by the unconstrained branch.
//!
//! The zero is deterministic, not floating-point luck. Over the [`LADDER_N`](super::LADDER_N) ladder the
//! squares `N²` are exact `f64` integers, and `Σ N²` is divisible by ten, so the centred means `x̄` and
//! `ȳ = 3x̄` are exact integers with no rounding. The centred normal equations are then exact integer
//! arithmetic: `Σ(x−x̄)(y−ȳ) = 3 Σ(x−x̄)²`, giving `b = 3` exactly, `a = ȳ − 3x̄ = 0` exactly, and every
//! residual `y − 3x = 0`, so `RSS = 0` exactly. The one platform assumption — that `N.powf(2.0)` returns
//! the exact square — holds on this target's correctly-rounded `pow`.

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::constrained_fit::constrained_fit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

/// The interior exponent under test — a grid node (`(2 + 38)/20 = 2.0`) at which the ladder is an exact
/// power law, so the constrained fit is exactly `(a, b) = (0, 3)`.
const EXACT_BETA: f64 = 2.0;

#[test]
fn accepts_exact_zero_intercept_fit() {
    let points = ladder_from(|n| 3.0 * n * n);

    // The deterministic core: at the exact exponent β = 2 the constrained solution is bit-exactly
    // (a, b, RSS) = (0, 3, 0), and it comes from the *unconstrained* branch — a = 0 is feasible under
    // a ≥ 0, so it is retained rather than projected onto the edge.
    let exact = constrained_fit(&points, EXACT_BETA)
        .expect("the exact power-law exponent admits a constrained fit");
    assert_eq!(exact.a(), 0.0, "the unconstrained intercept is exactly zero, got a = {}", exact.a());
    assert_eq!(exact.b(), 3.0, "the exact scale is the true coefficient b = 3, got {}", exact.b());
    assert_eq!(exact.rss(), 0.0, "an exact power law leaves exactly zero residual, got {}", exact.rss());

    // The optimizer selects that same exact candidate: β = 2 is a grid node whose RSS is the global
    // minimum 0, and the tie rule only unseats the incumbent on a *strictly lower* RSS (impossible below 0)
    // or an equal RSS at a smaller β (which would need an exact-zero residual at some β ≠ 2 — impossible on
    // this non-degenerate ladder). So `fit_block` returns the β = 2 grid candidate unchanged.
    let fit = fit_block(&points);
    let candidate = match fit {
        BlockFit::Identifiable(candidate) => candidate,
        other => panic!("a clean quadratic ladder must yield an identifiable interior fit, got {other:?}"),
    };
    assert_eq!(candidate.beta(), EXACT_BETA, "the optimizer selects the exact grid exponent β = 2");
    assert_eq!(candidate.a(), 0.0, "the selected fit retains the exact zero intercept, got a = {}", candidate.a());
    assert_eq!(candidate.b(), 3.0, "the selected fit recovers the exact scale b = 3, got {}", candidate.b());
    assert_eq!(candidate.rss(), 0.0, "the selected fit has exactly zero residual, got {}", candidate.rss());
}
