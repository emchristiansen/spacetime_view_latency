//! The complete durable identity of one attempt.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::attempt_ordinal::AttemptOrdinal;
use crate::indexed_sender_view_calibration_pilot::calibration_replicate::CalibrationReplicate;
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;
use crate::indexed_sender_view_calibration_pilot::calibration_target::CalibrationTarget;
use crate::indexed_sender_view_calibration_pilot::candidate_id::CandidateId;
use crate::indexed_sender_view_calibration_pilot::candidate_version::{
    CandidateVersion, INDEXED_SENDER_VIEW_CALIBRATION_VERSION,
};
use crate::indexed_sender_view_calibration_pilot::experiment_axis::ExperimentAxis;
use crate::indexed_sender_view_calibration_pilot::stage_repetition::StageRepetition;
use crate::plan::run_role::RunRole;

/// The complete identity of one attempt, per the spec's minimal type design: candidate, scale point,
/// role, attempt ordinal, and candidate version, so stale or mismatched results cannot join.
///
/// The role reuses the existing [`RunRole`] rather than minting a parallel vocabulary, and every
/// identity this module builds is minted at [`RunRole::Arm`] — there is no Control attempt, because
/// the calibration ceiling forbids the comparison one would exist for.
///
/// The ordinal is [`AttemptOrdinal::Original`] and can be nothing else, so the logical slot and the
/// full identity coincide here. [`Self::same_logical_slot`] is kept as an explicit method anyway,
/// because the inventory seal's duplicate check is stated over logical slots and should read the
/// same way it does in every other frozen inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct AttemptKey {
    candidate: CandidateId,
    axis: ExperimentAxis,
    rung: CalibrationRung,
    role: RunRole,
    stage: StageRepetition,
    ordinal: AttemptOrdinal,
    version: CandidateVersion,
}

impl AttemptKey {
    /// Mint the identity of one calibration replicate.
    ///
    /// **The replicate is the only parameter, and that is the point.** Six of the seven components
    /// are singletons of this freeze — one candidate, one axis, the baseline rung, the Arm role, the
    /// original ordinal, one candidate version — so accepting them would let a caller mint an
    /// identity the freeze does not contain. A general constructor taking a [`RunRole`] would in
    /// particular make a Control attempt representable, which is exactly the comparison the
    /// calibration ceiling forbids; there would then be a value in existence that the ceiling could
    /// only forbid by convention.
    ///
    /// With one parameter the inventory's exactness is nearly immediate: two replicates in, two
    /// distinct identities out, and no third identity is constructible at all. Nothing deserializes
    /// these, so no general constructor is needed for a round trip either.
    pub(crate) fn calibration(replicate: CalibrationReplicate) -> Self {
        Self {
            candidate: CandidateId::IndexedControlActivitySenderView,
            axis: ExperimentAxis::UnrelatedGlobalRows,
            rung: CalibrationRung::Baseline,
            role: RunRole::Arm,
            stage: StageRepetition::Calibration(replicate),
            ordinal: AttemptOrdinal::Original,
            version: INDEXED_SENDER_VIEW_CALIBRATION_VERSION,
        }
    }

    /// The endpoint this attempt runs at.
    pub(crate) fn rung(self) -> CalibrationRung {
        self.rung
    }

    /// The candidate this attempt exercises.
    pub(crate) fn candidate(self) -> CandidateId {
        self.candidate
    }

    /// Which replicate this attempt is.
    pub(crate) fn replicate(self) -> CalibrationReplicate {
        match self.stage {
            StageRepetition::Calibration(replicate) => replicate,
        }
    }

    /// The relation this attempt times.
    ///
    /// Always the measured arm: the composition witness is subscribed untimed after the batch and is
    /// never an attempt of its own.
    pub(crate) fn timed_target(self) -> CalibrationTarget {
        CalibrationTarget::MeasuredArm
    }

    /// Whether two identities address the same logical slot.
    pub(crate) fn same_logical_slot(self, other: Self) -> bool {
        self.candidate == other.candidate
            && self.axis == other.axis
            && self.rung == other.rung
            && self.role == other.role
            && self.stage == other.stage
            && self.version == other.version
    }
}

#[cfg(test)]
mod tests;
