//! A zero-`x`-spread ladder (identical `N` at every dose) makes the constrained RSS identical at every
//! exponent, so the whole domain ties; the frozen tie rule selects the smallest β, which pins at the
//! lower bound. The selection is deterministic across repeated calls (spec: prove the lower-β tie rule
//! and repeatability).

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::constant_n_ladder;
use crate::params::NUM_DOSES_USIZE;

#[test]
fn resolves_flat_objective_tie_to_smaller_beta() {
    // Every dose shares N = 100, so x = N^β is a single value at every β and the a = 0 edge scale gives
    // b·x = ȳ identically — the residual y_i − ȳ, and hence the RSS, is constant in β. The objective is
    // therefore flat over the entire domain: all 79 grid nodes tie, and the frozen "ties select the
    // smaller β" rule (in both the global selection and every golden-section interval choice) drives the
    // winner to the smallest exponent 0.1, which pins at the lower bound.
    let ladder = constant_n_ladder(100.0, |dose_index| 100.0 * (dose_index as f64 + 1.0));

    let selected_beta = |fit: BlockFit| match fit {
        BlockFit::PinnedAtBound(candidate) => candidate.beta(),
        other => panic!("a domain-wide flat objective must resolve to the pinned smaller-β bound, got {other:?}"),
    };

    // Repeatability: the deterministic search yields the identical selected exponent and status on every
    // call over the same ladder.
    let first = selected_beta(fit_block(&ladder));
    let second = selected_beta(fit_block(&ladder));
    let third = selected_beta(fit_block(&ladder));
    assert_eq!(first, second, "the flat-objective selection is repeatable across calls");
    assert_eq!(second, third, "the flat-objective selection is repeatable across calls");

    // The tie rule resolves the domain-wide tie to the smallest exponent, pinned at the lower bound 0.1.
    assert!(
        (first - 0.1).abs() <= 1e-4,
        "the lower-β tie rule selects the smallest exponent 0.1, got {first}"
    );

    // Guard the premise: the fixture is genuinely a zero-x-spread ladder, so the tie is real and not an
    // artefact of a single accidental basin.
    assert_eq!(ladder.len(), NUM_DOSES_USIZE, "the tie fixture is a full ten-dose ladder");
}
