//! Category proof: a run whose cumulative dose ladder is not strictly monotonic is rejected by the
//! private `validate_ladders` with typed adjacent-count evidence.

use super::staged_ladder::StagedLadder;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Dropping cell0/arm/block0's second dose's cumulative count below the first breaks strict monotonicity,
/// so `validate_ladders` fails with a typed `NonMonotonicLadder` pairing the previous and offending
/// cumulative counts at the perturbed run and dose.
#[test]
fn a_non_monotonic_ladder_is_rejected() {
    let previous_logical_n = DoseIndex::ALL[0].cumulative_driving_rows();
    let broken_logical_n = 0;

    let mut ladder = StagedLadder::canonical();
    // Break strict monotonicity at cell0/arm/block0's second dose: its count no longer exceeds the first.
    ladder.set_logical_n(0, RunRole::Arm, 0, 1, broken_logical_n);

    let Err(error) = super::super::validate_ladders(ladder.slots()) else {
        panic!("a non-monotonic dose ladder must fail validation");
    };

    match error {
        IntegrityError::NonMonotonicLadder {
            run,
            dose,
            previous_logical_n: previous,
            logical_n,
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the perturbed run"
            );
            assert_eq!(dose, DoseIndex::ALL[1], "the break is at the second dose");
            assert_eq!(
                previous, previous_logical_n,
                "the previous count is the first dose's cumulative count"
            );
            assert_eq!(
                logical_n, broken_logical_n,
                "the broken count is not strictly greater than the previous"
            );
        }
        other => panic!("expected NonMonotonicLadder, got {other:?}"),
    }
}
