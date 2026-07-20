//! Collection-order alignment proof: the secondary [`BetaDescriptor`]'s block index `i` and the primary
//! [`CellEvidence`]'s block index `i` denote the *same physical repetition block*, because both structures
//! independently sort by the same [`collection_order_key`](crate::analysis::validate::matched_block::MatchedBlock::collection_order_key).
//!
//! The proof is load-bearing because the collection order is deliberately *noncanonical*: construction
//! block `b` is placed at collection rank [`collection_rank`]`(b)` — a genuine permutation (multiplier 7 is
//! coprime to 30) that is not the identity — so array/construction order differs from collection order. Each
//! construction block also carries a *distinct* arm exponent [`arm_beta`]`(b)`, giving every block a unique
//! signature the descriptor recovers.
//!
//! At each collection position `i`: the primary evidence key is `2·i` (the key of the block placed at rank
//! `i`, since keys `{2·rank}` sorted ascending are `0, 2, …, 58`), and the descriptor block's identified
//! exponent is the exponent of that same construction block. Agreeing on both the collection key and the
//! per-block exponent pins descriptor index `i` and evidence index `i` to one construction block — proving
//! the two result arrays share one collection order rather than merely one length.

use super::N_BLOCKS;
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::params::BATCH_SIZE;
use crate::plan::run_role::RunRole;

/// The noncanonical collection rank of construction block `b`: `(7·b + 3) mod 30`. Multiplier `7` is
/// coprime to `30`, so this is a bijection of `0..30` onto itself — a genuine permutation, not the
/// identity, so construction order and collection order differ.
fn collection_rank(block: usize) -> usize {
    (7 * block + 3) % N_BLOCKS
}

/// The distinct arm exponent of construction block `b`: `1.0 + 0.05·b`, spanning `1.0 ..= 2.45` — all
/// strictly interior to `[0.1, 4.0]` and separated by `0.05`, far above the fit's identifiability
/// resolution, so each block's identified exponent is an unambiguous per-block signature.
fn arm_beta(block: usize) -> f64 {
    1.0 + 0.05 * block as f64
}

/// The flat direct-control median every dose carries: the control's own total change is zero, so the
/// control is valid and δ = this / 5. A rising arm above it is then Increasing.
const CONTROL_FLAT: u128 = 1_000_000;
/// The arm's power-law scale: every arm dose median is `CONTROL_FLAT + round(ARM_SCALE · N^β_b)`, large
/// enough that each block's paired-difference total change far exceeds δ and the cell is Increasing.
const ARM_SCALE: f64 = 100.0;

#[test]
fn descriptor_and_evidence_align_by_collection_order_key() {
    // Guard the premise: the collection order is genuinely noncanonical, so alignment cannot be an artefact
    // of construction order being preserved.
    assert!(
        (0..N_BLOCKS).any(|b| collection_rank(b) != b),
        "the collection order must be noncanonical for the alignment to be load-bearing"
    );

    let dataset = build_cell_dataset_fixture(collection_rank, arm_beta);
    let classified = BetaDescriptor::project(&dataset);
    let (evidence, beta) = match &classified {
        ClassifiedCell::Increasing { evidence, beta, .. } => (evidence, beta),
        other => panic!("a rising arm over a flat control must classify Increasing, got {other:?}"),
    };

    // The expected identified exponent at each *collection position*: construction block `b` lands at
    // position `collection_rank(b)`, carrying `arm_beta(b)`.
    let mut expected_beta_at_position = [f64::NAN; N_BLOCKS];
    for block in 0..N_BLOCKS {
        expected_beta_at_position[collection_rank(block)] = arm_beta(block);
    }

    let keys = evidence.collection_order_keys();
    let blocks = beta.blocks();
    for i in 0..N_BLOCKS {
        // Primary evidence is in ascending collection-key order: position `i` carries key `2·i`, the key of
        // the block placed at collection rank `i`.
        assert_eq!(
            keys[i].get(),
            2 * i as u64,
            "primary evidence position {i} carries the collection key of the block at rank {i}"
        );
        // The secondary descriptor block at that same position carries that block's identified exponent, so
        // descriptor index `i` and evidence index `i` denote the same physical block.
        let identified = blocks[i]
            .identified_beta()
            .expect("every clean power-law block is identifiable");
        assert!(
            (identified - expected_beta_at_position[i]).abs() < 0.01,
            "descriptor block {i} identified β {identified} matches exponent {} of the block at collection rank {i}",
            expected_beta_at_position[i]
        );
    }
}

/// Build the noncanonical-order cell dataset: control flat, arm a distinct power law per construction
/// block. Split out so the test body reads as the alignment argument rather than fixture plumbing.
fn build_cell_dataset_fixture(
    collection_rank: impl Fn(usize) -> usize,
    arm_beta: impl Fn(usize) -> f64,
) -> crate::analysis::validate::cell_dataset::CellDataset {
    CellDatasetFixture::build_with_collection_ranks(
        CellDatasetFixture::cell(),
        collection_rank,
        |block, role, dose| match role {
            RunRole::Control => CONTROL_FLAT,
            RunRole::Arm => {
                let n = ((dose as u64 + 1) * BATCH_SIZE) as f64;
                CONTROL_FLAT + (ARM_SCALE * n.powf(arm_beta(block))).round() as u128
            }
        },
    )
}
