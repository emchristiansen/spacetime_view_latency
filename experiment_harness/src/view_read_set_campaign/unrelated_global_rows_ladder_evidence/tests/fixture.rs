//! Shared construction: the frozen scale points, and one attempt identity over them.
//!
//! [`ScalePoint::ladder`] is the call the frozen inventory itself makes, so these rungs are the
//! preregistered ones rather than a stand-in.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::campaign_params::UNRELATED_GLOBAL_ROWS_LADDER_LEN;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

/// Every scale point of the frozen unrelated-global-rows ladder, ascending, as an array so a case
/// names its rungs by destructuring rather than by position.
pub(super) fn scales() -> [ScalePoint; UNRELATED_GLOBAL_ROWS_LADDER_LEN] {
    let ladder = ScalePoint::ladder(ExperimentAxis::UnrelatedGlobalRows);
    match <[ScalePoint; UNRELATED_GLOBAL_ROWS_LADDER_LEN]>::try_from(ladder.as_slice()) {
        Ok(scales) => scales,
        Err(_) => panic!(
            "the frozen unrelated-global-rows ladder has {UNRELATED_GLOBAL_ROWS_LADDER_LEN} rungs, \
             got {}",
            ladder.len()
        ),
    }
}

/// The Pilot's first two block indices, the coordinate a ladder may not mix.
pub(super) fn two_blocks() -> (PilotBlockIndex, PilotBlockIndex) {
    let [first, second, ..] = PilotBlockIndex::ALL;
    (first, second)
}

/// One Pilot attempt identity of the campaign's only candidate and version.
pub(super) fn key(
    scale: ScalePoint,
    role: RunRole,
    block: PilotBlockIndex,
    retry: RetryOrdinal,
) -> AttemptKey {
    AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        scale,
        role,
        StageRepetition::Pilot(block),
        retry,
        ENTITY_OWNER_SENDER_VIEW_VERSION,
    )
}
