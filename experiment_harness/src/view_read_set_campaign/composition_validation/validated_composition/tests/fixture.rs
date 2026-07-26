//! Shared fixture: one correct attempt's transition, phase-matched rows, and artifact directory.

use spacetimedb_sdk::Identity;

use crate::module_artifact::bindings::EntityOwner;
use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::campaign_params::{
    GLOBAL_KEY_BASE, OWNED_KEY_BASE, SEEDED_ROW_PAYLOAD,
};
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;
use crate::view_read_set_campaign::composition_validation::composition_transition_expectation::CompositionTransitionExpectation;
use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

use super::scratch_dir::scratch_dir;

/// The measured subscriber's identity, which owns the frozen ten-row slice.
pub(super) fn owned_owner() -> Identity {
    Identity::from_claims(
        "view-read-set-experiment-fixture",
        "composition-fixture-measured",
    )
}

/// The unrelated global identity, which owns the swept foreign slice. Distinct from
/// [`owned_owner`], which is what makes an owner mix-up detectable at all.
pub(super) fn foreign_owner() -> Identity {
    Identity::from_claims(
        "view-read-set-experiment-fixture",
        "composition-fixture-global",
    )
}

/// The ladder's first rung — one thousand foreign rows, the smallest frozen scale point.
pub(super) fn scale() -> ScalePoint {
    ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 0)
        .expect("the frozen unrelated-global-rows ladder has a first rung")
}

/// The phase-matched transition one attempt of `role` is judged against.
pub(super) fn transition(role: RunRole) -> CompositionTransitionExpectation {
    CompositionTransitionExpectation::required_final(scale(), role, owned_owner(), foreign_owner())
}

/// The saturated channel's schedule — the one the after phase is bound to.
pub(super) fn saturated_schedule() -> MutationSchedule {
    MutationSchedule::of(MeasurementChannel::SaturatedQueueGrowthPerWrite)
        .expect("the saturated channel issues measured writes, so it has a schedule")
}

/// What `role` correctly observes *before* any measured write: the whole owned slice seeded, plus —
/// for the Control's direct-table subscription only — the whole seeded foreign slice.
pub(super) fn seeded_rows(role: RunRole) -> Vec<EntityOwner> {
    let mut rows: Vec<EntityOwner> = saturated_schedule()
        .owned_slice_offsets()
        .into_iter()
        .map(|offset| EntityOwner {
            entity_uuid: owned_key(offset.get()),
            owner: owned_owner(),
            record: SEEDED_ROW_PAYLOAD.to_string(),
        })
        .collect();
    rows.extend(foreign_rows(role));
    rows
}

/// What `role` correctly observes once the saturated batch has confirmed: each owned key carrying
/// the schedule's derived final payload, the foreign slice still seeded.
pub(super) fn final_rows(role: RunRole) -> Vec<EntityOwner> {
    let schedule = saturated_schedule();
    let mut rows: Vec<EntityOwner> = schedule
        .owned_slice_offsets()
        .into_iter()
        .map(|offset| EntityOwner {
            entity_uuid: owned_key(offset.get()),
            owner: owned_owner(),
            record: schedule.final_payload_at_offset(offset),
        })
        .collect();
    rows.extend(foreign_rows(role));
    rows
}

/// The foreign slice as `role` observes it: the whole seeded slice for the Control's direct-table
/// subscription, nothing at all for the Arm's sender-scoped view.
fn foreign_rows(role: RunRole) -> Vec<EntityOwner> {
    match role {
        RunRole::Arm => Vec::new(),
        RunRole::Control => (0..scale().scale())
            .map(|index| EntityOwner {
                entity_uuid: GLOBAL_KEY_BASE
                    .checked_add(index)
                    .expect("the frozen foreign slice fits below the top of the key space"),
                owner: foreign_owner(),
                record: SEEDED_ROW_PAYLOAD.to_string(),
            })
            .collect(),
    }
}

/// One Pilot attempt's exclusively-created artifact directory, under a scratch root named by
/// `label`.
pub(super) fn attempt_directory(label: &str) -> AttemptArtifactDirectory {
    let root = scratch_dir(label);
    let attempt = AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        scale(),
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

/// Retain `rows` as this attempt's `label` observation, through the real persistence path.
pub(super) fn persist(
    directory: &AttemptArtifactDirectory,
    label: ObservedRowSetLabel,
    rows: Vec<EntityOwner>,
) -> ObservedRowSet {
    ObservedRowSet::persisted(directory, label, rows)
        .expect("the fixture's rows have distinct keys and line-break-free payloads")
}

/// The owned key at slice `offset`.
pub(super) fn owned_key(offset: u64) -> u64 {
    OWNED_KEY_BASE
        .checked_add(offset)
        .expect("an owned-slice offset names a key inside the frozen owned range")
}
