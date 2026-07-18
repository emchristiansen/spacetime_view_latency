//! Category proof: a matched block whose arm and control cumulative ladders disagree at a dose is
//! rejected by the private `validate_ladders` with typed arm/control evidence.

use super::staged_ladder::StagedLadder;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Perturbing cell0/block0's control first dose to one past the canonical count keeps every run
/// individually monotonic (BATCH_SIZE + 1 stays below the second dose's 2·BATCH_SIZE) but breaks the
/// block's arm/control agreement, so `validate_ladders` fails with a typed `CardinalityLadderMismatch`
/// pairing the arm and control cumulative counts at both run coordinates and the disagreeing dose.
#[test]
fn a_mismatched_cardinality_ladder_is_rejected() {
    let arm_logical_n = DoseIndex::ALL[0].cumulative_driving_rows();
    let control_logical_n = arm_logical_n + 1;

    let mut ladder = StagedLadder::canonical();
    ladder.set_logical_n(0, RunRole::Control, 0, 0, control_logical_n);

    let Err(error) = super::super::validate_ladders(ladder.slots()) else {
        panic!("a mismatched arm/control cardinality ladder must fail validation");
    };

    match error {
        IntegrityError::CardinalityLadderMismatch {
            arm,
            control,
            dose,
            arm_logical_n: arm_count,
            control_logical_n: control_count,
            ..
        } => {
            assert_eq!(
                arm,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the arm location is cell0/arm/block0"
            );
            assert_eq!(
                control,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Control, 0),
                "the control location is cell0/control/block0"
            );
            assert_eq!(dose, DoseIndex::ALL[0], "the disagreement is at the first dose");
            assert_eq!(arm_count, arm_logical_n, "the arm carries the canonical count");
            assert_eq!(
                control_count, control_logical_n,
                "the control carries the perturbed count"
            );
        }
        other => panic!("expected CardinalityLadderMismatch, got {other:?}"),
    }
}
