//! `SecondaryDescriptorReport::of` projects the descriptor's *sole* `block_search_outcomes()` sequence
//! faithfully — same 30-block cardinality, same per-block variant, same order — and mirrors its population
//! gating in both directions: `population` (the sole `Option`) is `Some` exactly when every block is
//! identifiable and `None` when they are not.
//!
//! Two Increasing datasets exercise both gate arms. Scenario A gives each block a *distinct* interior
//! exponent, so every block is `Identifiable`, the population is `Some`, and — because the per-block
//! content differs by position — the order comparison is load-bearing. Scenario B drives every block's
//! clean exponent to the frozen upper search bound, so every block is `PinnedAtBound` (non-identifiable)
//! and the population gate closes to `None`.

use serde_json::Value;

use super::super::N_BLOCKS;
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::report::block_search_report::BlockSearchReport;
use crate::analysis::report::secondary_descriptor_report::SecondaryDescriptorReport;
use crate::analysis::validate::cell_dataset::CellDataset;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::params::BATCH_SIZE;
use crate::plan::run_role::RunRole;

/// The flat direct-control median every dose carries: the control's own total change is zero, so the
/// control is valid and a rising arm above it is Increasing.
const CONTROL_FLAT: u128 = 1_000_000;
/// The arm's power-law scale: every arm dose median is `CONTROL_FLAT + round(ARM_SCALE · N^β)`.
const ARM_SCALE: f64 = 100.0;
/// The frozen inclusive upper bound of the exponent search domain (`fit_block::BETA_MAX`). A clean arm
/// with this true exponent lands at the boundary, so its constrained fit is `PinnedAtBound` — not an
/// identified estimate. Kept as a local literal because the source bound is `pub(super)` to the `beta`
/// module; the premise guard below asserts the bound-pinning actually occurred, so a drifted value fails
/// loud rather than silently weakening the test.
const BETA_UPPER_BOUND: f64 = 4.0;

/// Build an Increasing cell whose arm follows a clean power law with per-block exponent `arm_beta(block)`
/// over a flat control.
fn increasing_cell(arm_beta: impl Fn(usize) -> f64) -> CellDataset {
    CellDatasetFixture::build(
        CellDatasetFixture::cell(),
        move |block, role, dose| match role {
            RunRole::Control => CONTROL_FLAT,
            RunRole::Arm => {
                let n = ((dose as u64 + 1) * BATCH_SIZE) as f64;
                CONTROL_FLAT + (ARM_SCALE * n.powf(arm_beta(block))).round() as u128
            }
        },
    )
}

/// The `beta` descriptor of an Increasing classification, or a panic naming the actual variant.
fn increasing_descriptor(classified: &ClassifiedCell) -> &BetaDescriptor {
    match classified {
        ClassifiedCell::Increasing { beta, .. } => beta,
        other => {
            panic!("a rising power-law arm over a flat control must classify Increasing, got {other:?}")
        }
    }
}

/// Whether every element of a serialized block array carries the given externally-tagged `fit` variant —
/// serde renders `BlockFitReport` as a single-key object, so a matching variant is exactly that key.
fn every_fit_tagged(blocks: &Value, variant: &str) -> bool {
    blocks
        .as_array()
        .expect("the block reports serialize as an array")
        .iter()
        .all(|block| {
            block["fit"]
                .as_object()
                .map_or(false, |fit| fit.len() == 1 && fit.contains_key(variant))
        })
}

#[test]
fn mirrors_block_variant_order_and_population_gating() {
    // Scenario A — distinct interior exponents ⇒ every block Identifiable ⇒ population Some.
    let identifiable = increasing_cell(|block| 1.0 + 0.05 * block as f64);
    let classified_a = BetaDescriptor::project(&identifiable);
    let descriptor_a = increasing_descriptor(&classified_a);
    let report_a = SecondaryDescriptorReport::of(descriptor_a);

    // The fixed 30-block cardinality is a property of the report type.
    assert_eq!(report_a.blocks.len(), N_BLOCKS);

    // Sole-storage + order: the report's blocks are exactly the descriptor's own `block_search_outcomes()`
    // projected in sequence — no reorder, drop, or duplication. Distinct per-block exponents give each
    // position distinct serialized content, so a shuffle would change this array and be caught here.
    let blocks_a = serde_json::to_value(&report_a.blocks).expect("the block reports serialize");
    let expected_a: Vec<BlockSearchReport> = descriptor_a
        .block_search_outcomes()
        .iter()
        .map(BlockSearchReport::of)
        .collect();
    assert_eq!(
        blocks_a,
        serde_json::to_value(&expected_a).expect("the expected block reports serialize"),
        "the report stores exactly the descriptor's block_search_outcomes, projected in order"
    );

    // Every block projects as the `Identifiable` variant...
    assert!(
        every_fit_tagged(&blocks_a, "Identifiable"),
        "every interior-exponent block projects as an Identifiable fit"
    );
    // ...and the population interval — the sole Option — is Some, mirroring the source gate.
    assert_eq!(report_a.population.is_some(), descriptor_a.population().is_some());
    assert!(
        report_a.population.is_some(),
        "all-identifiable ⇒ a population β interval is derived and projected"
    );

    // Scenario B — a clean exponent at the frozen upper bound ⇒ every block PinnedAtBound ⇒ population None.
    let pinned = increasing_cell(|_| BETA_UPPER_BOUND);
    let classified_b = BetaDescriptor::project(&pinned);
    let descriptor_b = increasing_descriptor(&classified_b);
    let report_b = SecondaryDescriptorReport::of(descriptor_b);

    // Premise guard: the source really produced non-identifiable, bound-pinned blocks — so the None gate
    // below is load-bearing and not an artefact of a mis-tuned exponent that stayed identifiable.
    assert!(
        descriptor_b
            .block_search_outcomes()
            .iter()
            .all(|outcome| outcome.fit().is_pinned_at_bound()
                && outcome.fit().identified_beta().is_none()),
        "a bound exponent must yield non-identifiable (PinnedAtBound) blocks for the None gate to bind"
    );

    // Every block projects as the distinct `PinnedAtBound` variant — the variant mapping is faithful, not
    // collapsed to Scenario A's `Identifiable`.
    let blocks_b = serde_json::to_value(&report_b.blocks).expect("the block reports serialize");
    assert!(
        every_fit_tagged(&blocks_b, "PinnedAtBound"),
        "every bound-exponent block projects as a PinnedAtBound fit"
    );
    // The population Option is None, mirroring the source's closed gate exactly.
    assert!(descriptor_b.population().is_none());
    assert_eq!(report_b.population.is_some(), descriptor_b.population().is_some());
    assert!(
        report_b.population.is_none(),
        "a non-identifiable block ⇒ no population interval is derived or projected"
    );
}
