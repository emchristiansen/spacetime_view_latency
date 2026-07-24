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
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::record_seq::RecordSeq;
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

    // `mint` derives the run coordinate from the manifest it is handed; a fixture manifest built *for* the
    // control coordinate reports that control role, so the arm-position role check fails. The role check
    // fails before the manifest sequence is consulted, so any sequence serves here.
    let control_manifest =
        ValidatedRunManifest::fixture_for(control_coordinate.clone(), ScheduleSeed::new(0));
    let Err(error) = TrustedRun::<ArmRun>::mint(&control_manifest, doses, RecordSeq::zero()) else {
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
