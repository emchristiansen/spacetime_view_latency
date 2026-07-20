//! `CollectionOrderPlotReport::of` pairs each block total with its own collection-order key, position for
//! position in the evidence's schedule order, converts each total to milliseconds, and carries the frozen
//! `±δ` band. The proof drives a deliberately *noncanonical* collection order with a distinct total per
//! block, so a reorder or mispairing in the projection would break the exact serialized equality below.

use serde_json::json;

use super::super::N_BLOCKS;
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::finite_f64::FiniteF64;
use crate::analysis::report::collection_order_plot_report::CollectionOrderPlotReport;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::params::BATCH_SIZE;
use crate::plan::run_role::RunRole;

/// Noncanonical collection rank of construction block `b`: `(7·b + 3) mod 30`, a genuine permutation (7 is
/// coprime to 30), so construction order and collection order differ.
fn collection_rank(block: usize) -> usize {
    (7 * block + 3) % N_BLOCKS
}

/// A distinct rising arm exponent per construction block, so each block's total change is distinct — a
/// mispairing of totals to keys would therefore break the exact serialized equality below.
fn arm_beta(block: usize) -> f64 {
    1.0 + 0.05 * block as f64
}

/// The flat direct-control median every dose carries: the control's own total change is zero, so the
/// control is valid and a rising arm above it is Increasing.
const CONTROL_FLAT: u128 = 1_000_000;
/// The arm's power-law scale, large enough that each block's paired-difference total change is distinct.
const ARM_SCALE: f64 = 100.0;

#[test]
fn pairs_each_block_total_with_its_collection_order_key() {
    // Guard the premise: the collection order is genuinely noncanonical, so "preserves the evidence order"
    // is a real claim rather than an artefact of construction order being kept.
    assert!(
        (0..N_BLOCKS).any(|b| collection_rank(b) != b),
        "the collection order must be noncanonical for the pairing to be load-bearing"
    );

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
    let evidence = classified.evidence();

    let keys = evidence.collection_order_keys();
    let totals = evidence.arm_minus_control_totals();

    // A distinct total at every collection position, so mispairing any two positions would be observable.
    for i in 1..N_BLOCKS {
        assert_ne!(
            totals[i - 1].to_f64(),
            totals[i].to_f64(),
            "each collection position must carry a distinct total for the pairing to be load-bearing"
        );
    }

    // The projection must serialize exactly this: point `i` pairs the evidence's own key `i` with its own
    // total `i` rendered to milliseconds, in evidence order, plus the frozen margin as the `±δ` band.
    let expected_points: Vec<_> = (0..N_BLOCKS)
        .map(|i| {
            json!({
                "collection_order_key": keys[i].get(),
                "block_total_millis": FiniteF64::new(totals[i].to_f64() / 1_000_000.0).get(),
            })
        })
        .collect();
    let expected = json!({
        "points": expected_points,
        "delta_band_millis": FiniteF64::new(evidence.margin().to_millis_f64()).get(),
    });

    let report = CollectionOrderPlotReport::of(evidence);
    assert_eq!(
        serde_json::to_value(&report).expect("the plot report serializes"),
        expected,
        "each point pairs the block total with its own collection-order key, in evidence order, and the \
         band is the frozen margin in milliseconds"
    );
}
