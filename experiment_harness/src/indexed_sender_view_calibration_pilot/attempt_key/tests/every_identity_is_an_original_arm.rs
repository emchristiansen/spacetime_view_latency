//! Every identity this module can mint is an original Arm attempt at the frozen singletons.

use crate::indexed_sender_view_calibration_pilot::attempt_key::AttemptKey;
use crate::indexed_sender_view_calibration_pilot::attempt_ordinal::AttemptOrdinal;
use crate::indexed_sender_view_calibration_pilot::calibration_replicate::CalibrationReplicate;
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;
use crate::indexed_sender_view_calibration_pilot::candidate_id::CandidateId;
use crate::indexed_sender_view_calibration_pilot::experiment_axis::ExperimentAxis;
use crate::plan::run_role::RunRole;

/// Coverage: the six singleton components of an identity are fixed by the constructor and cannot be
/// varied by a caller, so a Control attempt and a non-baseline rung are unrepresentable here.
///
/// This matters because the calibration ceiling forbids the Arm/Control comparison a `RunRole` exists
/// for. `RunRole::Control` is a live variant crate-wide, so "no Control attempt" is not a property of
/// the role vocabulary — it is a property of this constructor being the only one, which is what this
/// test pins.
///
/// The assertions read private fields directly. This test is a child of the `attempt_key` module, so
/// it sees them without any accessor being added to the real API — the same discipline the accepted
/// screen's seal test uses. Reading them through `Serialize` instead would prove what the ledger says
/// rather than what the type holds, and it is the type that has to exclude the value.
#[test]
fn every_identity_is_an_original_arm() {
    for replicate in CalibrationReplicate::ALL {
        let key = AttemptKey::calibration(replicate);

        assert!(
            matches!(key.role, RunRole::Arm),
            "every calibration identity is minted at Arm; a Control attempt is the comparison the \
             ceiling forbids"
        );
        assert!(
            matches!(key.ordinal, AttemptOrdinal::Original),
            "this invocation schedules no retry, so every identity is an original"
        );
        assert!(
            matches!(key.rung, CalibrationRung::Baseline),
            "one rung only; a second would make the two replicates a low/high contrast"
        );
        assert!(
            matches!(key.candidate, CandidateId::IndexedControlActivitySenderView),
            "one candidate only"
        );
        assert!(
            matches!(key.axis, ExperimentAxis::UnrelatedGlobalRows),
            "the spec names unrelated/global rows as the required Site 4 axis"
        );

        // The replicate is the one coordinate the constructor takes, so it must survive into the
        // identity rather than being fixed alongside the rest.
        assert_eq!(
            key.replicate().get(),
            replicate.get(),
            "the varying coordinate must reach the identity"
        );
    }
}
