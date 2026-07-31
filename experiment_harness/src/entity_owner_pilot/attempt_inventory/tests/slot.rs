//! Shared fixture: the attempt identity this Pilot predeclares for one block and role.

use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::candidate_id::CandidateId;
use crate::entity_owner_pilot::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::entity_owner_pilot::experiment_axis::ExperimentAxis;
use crate::entity_owner_pilot::pilot_block_index::PilotBlockIndex;
use crate::entity_owner_pilot::retry_ordinal::RetryOrdinal;
use crate::entity_owner_pilot::stage_repetition::StageRepetition;
use crate::plan::run_role::RunRole;

/// The original attempt at one block's `role` slot.
///
/// Every component except the block and the role is fixed, because this single-axis Pilot walks one
/// candidate, one axis, and one stage — so these two are the whole coordinate a test needs to name.
/// `block` indexes [`PilotBlockIndex::ALL`], the sole source of block values.
pub(super) fn slot(block: usize, role: RunRole) -> AttemptKey {
    AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        ExperimentAxis::UnrelatedGlobalRows,
        role,
        StageRepetition::Pilot(PilotBlockIndex::ALL[block]),
        RetryOrdinal::ORIGINAL,
        ENTITY_OWNER_SENDER_VIEW_VERSION,
    )
}
