//! Adversarial join-by-key proof: under a deliberately noncanonical collection order, each serialized
//! secondary `BlockSearchReport` carries its own explicit `collection_order_key` *and keeps that key bound to
//! its own β outcome* — the serialized identified exponent under a given key is the exponent of the exact
//! construction block that owns that key. So a report consumer joins the β outcomes to the primary series by
//! the shared key, not by array position, and a buggy projection that emitted sorted keys while permuting the
//! outcomes beneath them would be caught (spec: temporal diagnostics "join ... β outcomes only by the proven
//! `collection_order_key`; no cross-sibling interpretation may depend on positional coincidence"; the proof
//! must "make noncanonical key order load-bearing and compare explicit keys, not merely array positions").
//!
//! The order is load-bearing two ways. Construction block `b` is placed at collection rank `(7·b + 3) mod 30`
//! — a genuine permutation (7 is coprime to 30) — so its collection key is `2·rank`, and the sorted key set is
//! `{0, 2, …, 58}` (values a positional index `0..30` renderer could never produce). And block `b` carries a
//! *distinct* arm exponent `arm_beta(b)`, so recovering the right exponent under each key pins that key to its
//! own construction block rather than to a same-length sibling that happens to share a sorted key sequence.

use std::collections::BTreeSet;

use serde_json::Value;

use super::super::N_BLOCKS;
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::report::collection_order_plot_report::CollectionOrderPlotReport;
use crate::analysis::report::secondary_descriptor_report::SecondaryDescriptorReport;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::params::BATCH_SIZE;
use crate::plan::run_role::RunRole;

/// The noncanonical collection rank of construction block `b`: `(7·b + 3) mod 30`, a genuine permutation
/// (7 is coprime to 30), so construction order and collection order differ.
fn collection_rank(block: usize) -> usize {
    (7 * block + 3) % N_BLOCKS
}

/// A distinct rising arm exponent per construction block, so every block is an identifiable Increasing fit
/// with a unique per-block signature and a full secondary descriptor exists.
fn arm_beta(block: usize) -> f64 {
    1.0 + 0.05 * block as f64
}

/// The flat direct-control median: the control's own total change is zero, so the control is valid and a
/// rising arm above it is Increasing.
const CONTROL_FLAT: u128 = 1_000_000;
/// The arm's power-law scale, large enough that each block's total change far exceeds δ.
const ARM_SCALE: f64 = 100.0;

/// The serialized `collection_order_key` at each index of a serialized array of key-bearing report objects
/// (the primary plot's `points` or the secondary descriptor's `blocks`).
fn keys_of(array: &Value) -> Vec<u64> {
    elements_of(array)
        .iter()
        .map(|element| {
            element["collection_order_key"]
                .as_u64()
                .expect("each element carries a numeric collection_order_key")
        })
        .collect()
}

/// The serialized array's elements, or a panic — shared by the key and β readers.
fn elements_of(array: &Value) -> &Vec<Value> {
    array
        .as_array()
        .expect("the report array serializes as a JSON array")
}

/// The identified exponent serialized under one secondary β block: `fit.Identifiable.candidate.beta`
/// (`FiniteF64` renders transparently as a bare number). Panics if the block is not an identifiable fit, so
/// a variant regression fails loud rather than silently skipping the binding check.
fn identified_beta_of(block: &Value) -> f64 {
    block["fit"]["Identifiable"]["candidate"]["beta"]
        .as_f64()
        .expect("each block is an Identifiable fit whose candidate carries a numeric β")
}

#[test]
fn carries_the_same_explicit_collection_keys_as_primary_points() {
    // Guard the premise: the collection order is genuinely noncanonical, so comparing keys is load-bearing
    // rather than an artefact of construction order being preserved.
    assert!(
        (0..N_BLOCKS).any(|b| collection_rank(b) != b),
        "the collection order must be noncanonical for the key comparison to be load-bearing"
    );

    // Invert the placement permutation: the construction block that lands at each collection rank. The block
    // whose serialized key is `2·rank` owns exponent `arm_beta(construction_at_rank[rank])`.
    let mut construction_at_rank = [usize::MAX; N_BLOCKS];
    for block in 0..N_BLOCKS {
        construction_at_rank[collection_rank(block)] = block;
    }

    let dataset = CellDatasetFixture::build_with_collection_ranks(
        CellDatasetFixture::cell(),
        collection_rank,
        |block, role, dose| match role {
            RunRole::Control => CONTROL_FLAT,
            RunRole::Arm => {
                let n = ((dose as u64 + 1) * BATCH_SIZE) as f64;
                CONTROL_FLAT + (ARM_SCALE * n.powf(arm_beta(block))).round() as u128
            }
        },
    );
    let classified = BetaDescriptor::project(&dataset);
    let (evidence, beta) = match &classified {
        ClassifiedCell::Increasing { evidence, beta, .. } => (evidence, beta),
        other => panic!("a rising power-law arm over a flat control must classify Increasing, got {other:?}"),
    };

    // Project both subtrees from the one classification and read their serialized explicit keys.
    let primary = serde_json::to_value(CollectionOrderPlotReport::of(evidence))
        .expect("the primary plot report serializes");
    let secondary = serde_json::to_value(SecondaryDescriptorReport::of(beta))
        .expect("the secondary descriptor report serializes");
    let primary_keys = keys_of(&primary["points"]);
    let secondary_keys = keys_of(&secondary["blocks"]);

    // The keys are the actual collection keys `{0, 2, …, 58}`, not positional indices `0..30` — so this
    // compares explicit keys a positional renderer could never reproduce.
    let expected_keys: Vec<u64> = (0..N_BLOCKS as u64).map(|rank| 2 * rank).collect();
    assert_eq!(
        primary_keys, expected_keys,
        "the primary points carry the sorted collection keys 0, 2, …, 58, not positional indices"
    );

    // The primary and secondary explicit key sequences agree — a necessary condition for a keyed join, but
    // not by itself proof of binding: a buggy projection could emit this same sorted sequence while permuting
    // the β outcomes beneath it. The per-key exponent check below is what pins each key to its own outcome.
    assert_eq!(
        secondary_keys, primary_keys,
        "the secondary β reports expose the same explicit collection-key sequence as the primary points"
    );

    // The load-bearing binding proof: for each serialized β block, map its *own* explicit key back through
    // the placement permutation to the construction block that owns it, and require the serialized identified
    // exponent to be that block's distinct `arm_beta`. Because every block's exponent is unique, an outcome
    // shuffled out from under its key would surface the wrong exponent here and fail — sorted keys alone
    // cannot rescue it.
    for block_report in elements_of(&secondary["blocks"]) {
        let key = block_report["collection_order_key"]
            .as_u64()
            .expect("each β block carries a numeric collection_order_key");
        assert_eq!(
            key % 2,
            0,
            "each collection key is `2·rank` for the block placed at that rank, so it is even; got {key}"
        );
        let rank = (key / 2) as usize;
        let construction_block = construction_at_rank[rank];
        let expected_beta = arm_beta(construction_block);
        let identified = identified_beta_of(block_report);
        assert!(
            (identified - expected_beta).abs() < 0.01,
            "the β block under collection key {key} must carry the exponent {expected_beta} of construction \
             block {construction_block} placed at rank {rank}, got {identified}"
        );
    }

    // Sanity: the binding check visited all 30 distinct keys, so no key was skipped or double-counted.
    assert_eq!(
        secondary_keys.iter().copied().collect::<BTreeSet<u64>>().len(),
        N_BLOCKS,
        "every one of the 30 distinct collection keys is present and checked"
    );
}
