//! Compile-time structural gate proof: the stored [`ClassifiedCell`] shape makes the primary→secondary
//! gate *unrepresentable*, which a runtime match alone cannot establish. [`assert_stored_shape`]
//! exhaustively destructures every variant with **every field named and no `..` rest pattern**, binding
//! each field to its exact type. That function compiling is the proof:
//!
//! - The `Increasing { evidence, comparison, beta }` arm binds `beta: &BetaDescriptor` — a *value*, not an
//!   `Option`. Were β made optional (`Option<BetaDescriptor>`) or removed, the type annotation or the field
//!   name would fail to compile: β can be neither absent nor optional on the Increasing branch.
//! - That same arm names *exactly* `evidence`, `comparison`, `beta`. Were an independent `response` field
//!   added to Increasing, the exhaustive (rest-less) pattern would fail to compile: an Increasing cell has
//!   no independently-variable response field — its response is definitionally Increasing.
//! - The `NonIncreasing` arm binds `response: &NonIncreasingResponse` and has *no* `beta` field, and
//!   [`NonIncreasingResponse`] has no `Increasing` variant, so "non-Increasing with a descriptor" and
//!   "Increasing hiding in a non-Increasing response" are both unrepresentable.
//!
//! The `#[test]` then projects a real Increasing dataset and binds its `beta` to a `&BetaDescriptor`, so
//! the mandatory-descriptor shape is exercised at runtime as well.

use super::{arm_median, CONTROL_FLAT_NANOS, FLAT_CONTROL_DELTA};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::classify::non_increasing_response::NonIncreasingResponse;
use crate::analysis::classify::prediction_comparison::PredictionComparison;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::plan::run_role::RunRole;

/// The per-dose arm rise that makes the projected cell Increasing (total change `9 · 100 000` ns, above
/// the 200 000 ns band), so the `#[test]` exercises the mandatory-descriptor branch at runtime.
const PER_DOSE: i128 = 100_000;

/// Exhaustively destructure the stored classification, naming every field of every variant with no rest
/// pattern and binding each to its exact type. This function *compiling* is the structural proof; its body
/// only consumes the bindings so they are load-bearing. It is never called for effect.
fn assert_stored_shape(cell: &ClassifiedCell) {
    match cell {
        ClassifiedCell::InvalidControl { evidence } => {
            let _evidence: &CellEvidence = evidence;
        }
        ClassifiedCell::NonIncreasing {
            evidence,
            response,
            comparison,
        } => {
            let _evidence: &CellEvidence = evidence;
            // A non-Increasing response, drawn from a taxonomy with no Increasing variant, and no β field.
            let _response: &NonIncreasingResponse = response;
            let _comparison: &PredictionComparison = comparison;
        }
        ClassifiedCell::Increasing {
            evidence,
            comparison,
            beta,
        } => {
            let _evidence: &CellEvidence = evidence;
            let _comparison: &PredictionComparison = comparison;
            // Mandatory and non-optional: `&BetaDescriptor`, not `&Option<BetaDescriptor>`. No `response`
            // field is nameable here — the exhaustive rest-less pattern above would not compile if one existed.
            let _beta: &BetaDescriptor = beta;
        }
    }
}

#[test]
fn the_stored_shape_forbids_optional_or_independent_response() {
    let dataset = CellDatasetFixture::build(CellDatasetFixture::cell(), |_block, role, dose| match role {
        RunRole::Control => CONTROL_FLAT_NANOS,
        RunRole::Arm => arm_median(PER_DOSE, dose),
    });
    let classified = BetaDescriptor::project(&dataset);

    // Exercise the exhaustive-shape assertion on a real value, then bind the mandatory descriptor directly.
    assert_stored_shape(&classified);
    match classified {
        ClassifiedCell::Increasing { beta, .. } => {
            let _mandatory: &BetaDescriptor = &beta;
        }
        other => panic!("the shaped Increasing dataset must project to the Increasing branch, got {other:?}"),
    }

    // The band the fixture targets, restated so the runtime instance is a genuine Increasing cell.
    assert!((9 * PER_DOSE) > FLAT_CONTROL_DELTA as i128, "the fixture arm total lands above +δ");
}
