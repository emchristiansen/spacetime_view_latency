//! The graph-level aggregation smoke: the validation-owned complete-campaign fixture folds into the
//! trusted graph, and classifying it exercises the real per-block/per-cell aggregation and gate.
//!
//! Every run's every dose carries the identical `1..=BATCH_SIZE` latency vector, so the R-1 per-dose
//! median is a constant 500 ns everywhere. Both the arm-minus-control paired difference `D(N)` and the
//! direct control's response are therefore flat: each block's Theil–Sen slope is zero, so every arm and
//! control total change is zero and both exact 30-block intervals are the degenerate `[0, 0]`. The
//! frozen margin is `δ = median(500 ns) / 5 = 100 ns`, so `[0, 0]` lies within `[-δ, +δ]`: every cell's
//! control is valid and every arm response is Flat-equivalent. The prediction comparison then falls out
//! of each cell's preregistered prediction — Confirmed for the Flat-predicting key-scoped-unrelated
//! arms, Contradicted for the Increasing-predicting table-scoped and own-slice arms.

use crate::analysis::classify::cell_classification::CellClassification;
use crate::analysis::classify::classified_campaign::ClassifiedCampaign;
use crate::analysis::classify::prediction_comparison::PredictionComparison;
use crate::analysis::classify::response_class::ResponseClass;
use crate::analysis::validate::validated_campaign::tests::campaign_builder::CampaignFixture;
use crate::analysis::validate::validated_campaign::ValidatedCampaign;
use crate::plan::cell::Cell;
use crate::plan::predicted_response::PredictedResponse;

#[test]
fn a_uniform_campaign_classifies_every_cell_flat_equivalent() {
    let campaign = ValidatedCampaign::from_records(CampaignFixture::valid().into_records())
        .expect("the complete, spec-correct campaign must fold without an integrity error");
    let classified = ClassifiedCampaign::classify(&campaign);

    let all_cells = Cell::all();
    assert_eq!(
        classified.cells().len(),
        all_cells.len(),
        "the classifier emits exactly one classification per preregistered cell"
    );

    for (index, classification) in classified.cells().iter().enumerate() {
        let cell = all_cells[index];
        match classification {
            CellClassification::Valid {
                evidence,
                response,
                comparison,
            } => {
                assert_eq!(
                    evidence.cell(),
                    cell,
                    "classifications are emitted in canonical Cell::all() order"
                );
                assert_eq!(
                    *response,
                    ResponseClass::FlatEquivalent,
                    "a flat arm response is Flat-equivalent within the frozen band"
                );
                // The observed response is Flat-equivalent for every cell, so a Flat prediction is
                // Confirmed and an Increasing prediction is Contradicted.
                let expected = match cell.predicted_response() {
                    PredictedResponse::FlatIn(_) => PredictionComparison::Confirmed,
                    PredictedResponse::IncreasingIn(_) => PredictionComparison::Contradicted,
                };
                assert_eq!(
                    *comparison, expected,
                    "the flat observation is compared against the cell's preregistered prediction"
                );
            }
            CellClassification::InvalidControl { .. } => panic!(
                "the uniform control response is flat and within the band, so every cell is valid"
            ),
        }
    }
}
