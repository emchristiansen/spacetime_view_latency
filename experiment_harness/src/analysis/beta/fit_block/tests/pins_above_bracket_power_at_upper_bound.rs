//! An above-bracket power ladder `y = N^5`, whose true exponent lies beyond the search domain, drives
//! the objective toward the upper bound, so the search pins at `β = 4.0` with [`BlockFit::PinnedAtBound`]
//! — retained, non-identifiable (spec: mark an above-bracket power as pinned).

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

#[test]
fn pins_above_bracket_power_at_upper_bound() {
    // y = N^5 has true exponent 5, above the [0.1, 4.0] domain. RSS decreases monotonically as β climbs
    // toward the truth, so the constrained minimum over the domain is at the upper bound 4.0.
    let fit = fit_block(&ladder_from(|n| n.powi(5)));

    let candidate = match fit {
        BlockFit::PinnedAtBound(candidate) => candidate,
        other => panic!("an above-bracket power must pin at the upper domain bound, got {other:?}"),
    };
    assert!(fit.is_pinned_at_bound(), "the above-bracket outcome is bound-pinned");
    assert!(!fit.is_identifiable(), "a bound-pinned outcome is never identifiable");
    assert!(
        (candidate.beta() - 4.0).abs() <= 1e-4,
        "the selected exponent pins within 1e-4 of the upper bound 4.0, got {}",
        candidate.beta()
    );
}
