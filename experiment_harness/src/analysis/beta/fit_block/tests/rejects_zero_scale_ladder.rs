//! An all-zero-latency ladder carries no positive power-law scale at any exponent, so the fit is
//! rejected as [`BlockFit::NonPositiveScale`] (spec: reject non-positive-scale data; "no fit exists when
//! no candidate has strictly positive finite `b`").

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

#[test]
fn rejects_zero_scale_ladder() {
    // Every y = 0 (the nonnegative floor). At every β the only constrained fit has b = 0, which is not
    // strictly positive, so no candidate exists anywhere in the domain.
    let fit = fit_block(&ladder_from(|_n| 0.0)).fit();

    assert!(
        matches!(fit, BlockFit::NonPositiveScale),
        "an all-zero ladder has no positive-scale fit and must be NonPositiveScale, got {fit:?}"
    );
    assert!(
        !fit.is_identifiable(),
        "a non-positive-scale outcome is never identifiable"
    );
    assert!(
        fit.candidate().is_none(),
        "NonPositiveScale structurally retains no numeric candidate"
    );
}
