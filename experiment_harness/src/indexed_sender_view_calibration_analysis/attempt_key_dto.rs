//! The untrusted wire form of one attempt's durable identity.

use serde::Deserialize;

use crate::indexed_sender_view_calibration_analysis::attempt_ordinal_dto::AttemptOrdinalDto;
use crate::indexed_sender_view_calibration_analysis::calibration_rung_dto::CalibrationRungDto;
use crate::indexed_sender_view_calibration_analysis::candidate_id_dto::CandidateIdDto;
use crate::indexed_sender_view_calibration_analysis::experiment_axis_dto::ExperimentAxisDto;
use crate::indexed_sender_view_calibration_analysis::frozen_candidate_version::frozen_candidate_version;
use crate::indexed_sender_view_calibration_analysis::run_role_dto::RunRoleDto;
use crate::indexed_sender_view_calibration_analysis::stage_repetition_dto::StageRepetitionDto;

/// The wire form of
/// [`AttemptKey`](crate::indexed_sender_view_calibration_pilot::attempt_key::AttemptKey).
///
/// **Every component is decoded and every component is checked.** An earlier draft retained the
/// single-valued components verbatim, reasoning that their source enums have one variant each. That
/// reasoning was wrong in a way worth recording: a one-variant enum constrains what the *pilot
/// writes*, and this analyzer reads a file. A field that is never deserialized is never validated, so
/// that draft would have admitted a line naming any candidate, any axis, any rung, any role — a
/// `Control` line included — and any ordinal, because nothing would ever have looked.
///
/// The seven components are therefore mirrored as exact-variant DTOs and matched against the freeze
/// by [`Self::matches_frozen`], and the version is compared against the pilot's own constant rather
/// than a restated literal.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttemptKeyDto {
    pub(crate) candidate: CandidateIdDto,
    pub(crate) axis: ExperimentAxisDto,
    pub(crate) rung: CalibrationRungDto,
    pub(crate) role: RunRoleDto,
    /// The only component that varies across this freeze: which replicate this attempt is.
    pub(crate) stage: StageRepetitionDto,
    pub(crate) ordinal: AttemptOrdinalDto,
    /// The candidate version, so a series from a different pinned artifact cannot silently join.
    pub(crate) version: u32,
}

impl AttemptKeyDto {
    /// Whether this identity is one the freeze can actually contain.
    ///
    /// Destructures `Self`, so every component this DTO decodes must participate: adding a field
    /// here without deciding what it means is a compile error. As with
    /// [`MethodFactsDto`](super::method_facts_dto::MethodFactsDto), that is a guarantee about *this*
    /// type and not across the serde boundary — the pilot growing a key component is caught at
    /// runtime by `deny_unknown_fields`, loudly, not at build time.
    ///
    /// The replicate ordinal is deliberately **not** checked here.
    /// [`ReplicatePair`](super::replicate_pair::ReplicatePair) checks the ordinal *set* against the
    /// frozen inventory, which is a statement about the two records together that no single record
    /// can make.
    pub(crate) fn matches_frozen(self) -> bool {
        let Self {
            candidate,
            axis,
            rung,
            role,
            // Checked as a set by `ReplicatePair`, not per record — see this method's doc.
            stage: _,
            ordinal,
            version,
        } = self;
        candidate == CandidateIdDto::IndexedControlActivitySenderView
            && axis == ExperimentAxisDto::UnrelatedGlobalRows
            && rung == CalibrationRungDto::Baseline
            && role == RunRoleDto::Arm
            && ordinal == AttemptOrdinalDto::Original
            && version == frozen_candidate_version()
    }

    /// The 0-based replicate index this record belongs to.
    pub(crate) fn replicate(self) -> u32 {
        self.stage.replicate()
    }
}
