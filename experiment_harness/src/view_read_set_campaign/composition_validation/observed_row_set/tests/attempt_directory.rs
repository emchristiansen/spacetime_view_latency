//! Shared fixture: one attempt's real artifact directory under a fresh scratch root.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

use super::scratch_dir::scratch_dir;

/// The exclusively-created artifact directory of one Pilot attempt, under a scratch root named by
/// `label`.
///
/// The real type rather than a bare path, because it is the only thing
/// [`ObservedRowSet::persisted`](crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet::persisted)
/// accepts — which is the point of it being typed.
pub(super) fn attempt_directory(label: &str) -> AttemptArtifactDirectory {
    let root = scratch_dir(label);
    let scale = ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 0)
        .expect("the frozen unrelated-global-rows ladder has a first rung");
    let attempt = AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        scale,
        RunRole::Arm,
        StageRepetition::Pilot(
            PilotBlockIndex::ALL
                .first()
                .copied()
                .expect("the Pilot stage has at least one frozen block"),
        ),
        RetryOrdinal::ORIGINAL,
        ENTITY_OWNER_SENDER_VIEW_VERSION,
    );
    AttemptArtifactDirectory::create(&root, attempt)
        .expect("a fresh scratch root and an unused attempt identity create the directory")
}
