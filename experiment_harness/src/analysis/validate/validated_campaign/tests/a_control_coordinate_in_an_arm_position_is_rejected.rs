//! Category proof: minting a control-role coordinate into an arm position is rejected by
//! `TrustedRun::mint` with typed role evidence.
//!
//! This is the defense-in-depth role check the canonical fold can never trip (the fold always mints a
//! coordinate whose role matches the position), so it is proven directly through the minting path rather
//! than through `from_records`.

use super::staged_ladder::StagedLadder;
use crate::analysis::validate::arm_run::ArmRun;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::trusted_run::TrustedRun;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// A complete, well-formed control dose ladder minted into an arm position fails `TrustedRun::mint`'s
/// role check, so it returns a typed `RunRoleMismatch` carrying the control-role coordinate and the arm
/// role the position expected.
#[test]
fn a_control_coordinate_in_an_arm_position_is_rejected() {
    let cell = Cell::all()[0];
    let control_coordinate = super::super::canonical_coordinate(cell, RunRole::Control, 0);
    let doses = StagedLadder::trusted_doses(cell);

    let Err(error) = TrustedRun::<ArmRun>::mint(control_coordinate.clone(), doses) else {
        panic!("minting a control-role coordinate into an arm position must fail");
    };

    match error {
        IntegrityError::RunRoleMismatch {
            coordinate,
            expected,
            ..
        } => {
            assert_eq!(
                coordinate, control_coordinate,
                "the offending coordinate carries the control role"
            );
            assert_eq!(
                expected,
                RunRole::Arm,
                "the arm position expected an arm-role coordinate"
            );
        }
        other => panic!("expected RunRoleMismatch, got {other:?}"),
    }
}
