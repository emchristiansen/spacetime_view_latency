//! The control-validity gate is structural: an in-band control interval mints `Valid` with the arm's
//! response and prediction comparison, while an out-of-band control interval mints `InvalidControl`
//! carrying no arm claim.

use crate::analysis::classify::cell_classification::CellClassification;
use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::classify::equivalence_margin::{CONTROL_DOSE_MEDIAN_COUNT, EquivalenceMargin};
use crate::analysis::classify::prediction_comparison::PredictionComparison;
use crate::analysis::classify::response_class::ResponseClass;
use crate::analysis::stats::rational::Rational;
use crate::observation::record_seq::RecordSeq;
use crate::plan::cell::Cell;
use crate::plan::table_scoped_arm::TableScopedArm;

/// Per-cell evidence with every arm block-total at `arm_total`, every control block-total at
/// `control_total`, and a margin frozen to the integer `delta`. Uniform totals put each exact interval
/// at the degenerate point `[arm_total, arm_total]` / `[control_total, control_total]`, so the gate and
/// response are driven by chosen constants; the collection-order keys are distinct but irrelevant here.
fn evidence(cell: Cell, arm_total: i128, control_total: i128, delta: i128) -> CellEvidence {
    let arm_minus_control_totals: [Rational; 30] =
        std::array::from_fn(|_| Rational::from_int(arm_total));
    let control_totals: [Rational; 30] = std::array::from_fn(|_| Rational::from_int(control_total));
    let collection_order_keys: [RecordSeq; 30] = std::array::from_fn(|i| RecordSeq::new(i as u64));
    let control_dose_medians: [Rational; CONTROL_DOSE_MEDIAN_COUNT] =
        std::array::from_fn(|_| Rational::from_int(delta * 5));
    let margin = EquivalenceMargin::freeze(&control_dose_medians);
    CellEvidence::new(
        cell,
        arm_minus_control_totals,
        control_totals,
        collection_order_keys,
        margin,
    )
}

#[test]
fn gates_the_arm_claim_on_control_validity() {
    // Arm A under unrelated growth predicts Increasing.
    let cell = Cell::TableScopedUnrelated(TableScopedArm::ProceduralRange);

    // Valid: the control interval [0, 0] lies within [-100, +100]; the arm interval [1000, 1000] has its
    // lower bound above +δ → Increasing; the cell predicted Increasing → Confirmed.
    match CellClassification::classify(evidence(cell, 1000, 0, 100)) {
        CellClassification::Valid {
            response,
            comparison,
            ..
        } => {
            assert_eq!(
                response,
                ResponseClass::Increasing,
                "the arm interval above +δ is Increasing"
            );
            assert_eq!(
                comparison,
                PredictionComparison::Confirmed,
                "an Increasing observation confirms the cell's Increasing prediction"
            );
        }
        CellClassification::InvalidControl { .. } => {
            panic!("a control interval within the band is valid")
        }
    }

    // InvalidControl: the control interval [1000, 1000] lies outside [-100, +100]; the whole cell is
    // invalidated and exposes no arm response or comparison.
    assert!(
        matches!(
            CellClassification::classify(evidence(cell, 0, 1000, 100)),
            CellClassification::InvalidControl { .. }
        ),
        "a control interval outside the band invalidates the whole cell"
    );
}
