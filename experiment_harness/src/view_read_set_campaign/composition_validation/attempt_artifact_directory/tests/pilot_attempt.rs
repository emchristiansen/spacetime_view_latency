//! Shared fixture: one Pilot attempt identity, varying only the coordinates these tests vary.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

/// A Pilot attempt identity at the ladder's first rung, on `block`, as the Arm's original attempt.
///
/// The block is the parameter because it is what distinguishes two otherwise-identical attempts,
/// which is the collision the per-attempt directory exists to keep apart.
pub(super) fn pilot_attempt(block: usize) -> AttemptKey {
    let scale = ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 0)
        .expect("the frozen unrelated-global-rows ladder has a first rung");
    AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        scale,
        RunRole::Arm,
        StageRepetition::Pilot(
            PilotBlockIndex::ALL
                .get(block)
                .copied()
                .expect("the requested test block is one of the five frozen Pilot blocks"),
        ),
        RetryOrdinal::ORIGINAL,
        ENTITY_OWNER_SENDER_VIEW_VERSION,
    )
}
