//! A constant-latency ladder `y = 500` drives the objective monotonically toward the flattest exponent,
//! so the search pins at the lower domain bound `β = 0.1` with [`BlockFit::PinnedAtBound`] — retained,
//! non-identifiable (spec: a candidate within `1e-4` of a bracket edge is `PinnedAtBound` and remains
//! non-identifiable).

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

#[test]
fn pins_constant_ladder_at_lower_bound() {
    // A constant y has no centred x-covariance, so every fit is the a = 0 edge b = Σy·x/Σx²; its RSS
    // shrinks monotonically as β → 0 flattens x = N^β toward a constant. The minimum over [0.1, 4.0] is
    // at the lower bound, so the selected exponent pins near 0.1.
    let fit = fit_block(&ladder_from(|_n| 500.0)).fit();

    let candidate = match fit {
        BlockFit::PinnedAtBound(candidate) => candidate,
        other => panic!("a constant ladder must pin at the lower domain bound, got {other:?}"),
    };
    assert!(fit.is_pinned_at_bound(), "the constant-ladder outcome is bound-pinned");
    assert!(!fit.is_identifiable(), "a bound-pinned outcome is never identifiable");
    assert!(
        (candidate.beta() - 0.1).abs() <= 1e-4,
        "the selected exponent pins within 1e-4 of the lower bound 0.1, got {}",
        candidate.beta()
    );
}
