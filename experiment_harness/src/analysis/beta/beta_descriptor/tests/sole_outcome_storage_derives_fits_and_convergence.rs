//! The descriptor's *sole* per-block storage is the 30 [`BlockSearchOutcome`]s: the authoritative fits it
//! exposes are *derived* from that one storage rather than held separately, so a fit can never disagree
//! with its own convergence record (spec: sole per-block descriptor storage; migrate callers to
//! iterator/indexed fit access without duplicated fit state). This proof drives the real per-block fit
//! over 30 clean linear ladders and shows the copied-fit iterator is pointwise identical to the stored
//! outcomes' fits, and that the same single storage carries each block's convergence record.

use crate::analysis::beta::basin_selection::BasinSelection;
use crate::analysis::beta::beta_descriptor::tests::{ladder_from, N_BLOCKS};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::beta::block_point::BlockPoint;
use crate::params::NUM_DOSES_USIZE;

#[test]
fn sole_outcome_storage_derives_fits_and_convergence() {
    let block = ladder_from(|n| 2.0 * n);
    let ladders: [[BlockPoint; NUM_DOSES_USIZE]; N_BLOCKS] = [block; N_BLOCKS];
    let descriptor = BetaDescriptor::from_ladders(&ladders);

    let outcomes = descriptor.block_search_outcomes();
    assert_eq!(outcomes.len(), N_BLOCKS, "the sole storage holds one outcome per block");

    // The copied-fit iterator is derived from the sole outcome storage: pointwise identical to each stored
    // outcome's fit, block for block. There is no separate fit array that could drift from the outcomes.
    let derived_fits: Vec<_> = descriptor.block_fits().collect();
    assert_eq!(derived_fits.len(), N_BLOCKS, "the derived fit iterator spans every block");
    for (i, fit) in derived_fits.iter().enumerate() {
        assert_eq!(
            fit.identified_beta(),
            outcomes[i].fit().identified_beta(),
            "derived block_fits()[{i}] is exactly the stored outcome's fit"
        );
    }

    // The same single storage carries each block's convergence record alongside its fit: every clean
    // linear block selected a basin, so its outcome's convergence names a Selected basin whose candidate
    // equals the stored fit's candidate (enforced at the BlockSearchOutcome mint boundary). Reading the
    // fit and the convergence from one indexed outcome is the migrated indexed-access path.
    for (i, outcome) in outcomes.iter().enumerate() {
        assert!(
            matches!(outcome.convergence().selection(), BasinSelection::Selected { .. }),
            "block {i}'s outcome carries the convergence of the search that produced its fit"
        );
    }
}
