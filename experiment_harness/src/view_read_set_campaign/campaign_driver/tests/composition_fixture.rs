//! Shared fixture: one role's correct measured attempt — its identity, its two phase-matched
//! observations retained through the real persistence path, and a measured channel prefix.
//!
//! Copied from the composition check's own fixture rather than shared with it. That tree's helpers
//! are `pub(super)` and reach only its tests, and the duplication is the same deliberate one the
//! writer fixtures already carry: a shared fixture would couple the census's tests to the driver
//! stage's, so a convenience change in one tree would silently move the other's inputs.
//!
//! Every value comes from a real constructor. The rows are built from the frozen schedule's own walk
//! rather than typed out, the artifacts are written and digested by [`ObservedRowSet::persisted`],
//! and each [`ChannelEvidence`] performs its own reduction — nothing here fabricates a finding or a
//! statistic.

use std::time::Duration;

use spacetimedb_sdk::Identity;

use crate::module_artifact::bindings::EntityOwner;
use crate::observation::latency_sample::LatencySample;
use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::campaign_driver::MeasuredFailure;
use crate::view_read_set_campaign::campaign_params::{
    GLOBAL_KEY_BASE, OWNED_KEY_BASE, SEEDED_ROW_PAYLOAD,
};
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;
use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;
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
    Identity::from_claims("view-read-set-driver-fixture", "driver-fixture-measured")
}

/// The unrelated global identity, which owns the swept foreign slice. Distinct from
/// [`owned_owner`], which is what makes an owner mix-up detectable at all.
pub(super) fn foreign_owner() -> Identity {
    Identity::from_claims("view-read-set-driver-fixture", "driver-fixture-global")
}

/// The ladder's first rung — one thousand foreign rows, the smallest frozen scale point.
pub(super) fn scale() -> ScalePoint {
    ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 0)
        .expect("the frozen unrelated-global-rows ladder has a first rung")
}

/// One attempt identity in `role`, at the fixture's scale point.
///
/// The stage mints its expectation from this key's own scale and role, so the identity is what binds
/// the transition to the rows below — handing it a key of the other role is exactly the mismatch the
/// refusal test relies on.
pub(super) fn key(role: RunRole) -> AttemptKey {
    AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        scale(),
        role,
        StageRepetition::Pilot(
            PilotBlockIndex::ALL
                .first()
                .copied()
                .expect("the Pilot stage has five frozen blocks"),
        ),
        RetryOrdinal::ORIGINAL,
        ENTITY_OWNER_SENDER_VIEW_VERSION,
    )
}

/// The saturated channel's schedule — the one the after phase is bound to.
fn saturated_schedule() -> MutationSchedule {
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

/// One row of the foreign slice, for the test that leaks exactly one into the Arm's view.
pub(super) fn one_foreign_row() -> EntityOwner {
    EntityOwner {
        entity_uuid: GLOBAL_KEY_BASE,
        owner: foreign_owner(),
        record: SEEDED_ROW_PAYLOAD.to_string(),
    }
}

/// One attempt's exclusively-created artifact directory, under a scratch root named by `label`.
pub(super) fn attempt_directory(label: &str, role: RunRole) -> AttemptArtifactDirectory {
    let root = scratch_dir(label);
    AttemptArtifactDirectory::create(&root, key(role))
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
fn owned_key(offset: u64) -> u64 {
    OWNED_KEY_BASE
        .checked_add(offset)
        .expect("an owned-slice offset names a key inside the frozen owned range")
}

/// A measured channel prefix: the two single-sample apply channels, each reduced by its own
/// constructor.
///
/// Two rather than four because the stage neither counts the prefix nor inspects it — it moves it
/// through — so what a test needs from it is that it is non-empty, ordered, and identifiable on the
/// way out. Channel completeness is [`ScalePointEvidence`](crate::view_read_set_campaign::scale_point_evidence::ScalePointEvidence)'s
/// to enforce, and it does so separately.
pub(super) fn measured_prefix() -> Vec<ChannelEvidence> {
    vec![
        ChannelEvidence::cold_subscription(LatencySample::from_elapsed(Duration::from_millis(7)))
            .expect("a seven-millisecond cold apply is a strictly positive duration"),
        ChannelEvidence::reconnect(LatencySample::from_elapsed(Duration::from_millis(3)))
            .expect("a three-millisecond reconnect apply is a strictly positive duration"),
    ]
}

/// Require the stage to have accepted a composition, reporting the refusal itself when it did not.
///
/// `Result::expect` is unavailable here: it would need [`MeasuredFailure`] to be `Debug`, and adding
/// that derive to production for a test's convenience buys a strictly worse message than rendering
/// the census's own error, which is what actually says *why* a fixture stopped being correct.
pub(super) fn accepted(
    outcome: std::result::Result<(Vec<ChannelEvidence>, ValidatedComposition), MeasuredFailure>,
) -> (Vec<ChannelEvidence>, ValidatedComposition) {
    match outcome {
        Ok(validated) => validated,
        Err(failure) => panic!(
            "this fixture's rows are the composition its role requires, but the stage refused \
             them: {:#}",
            failure.error
        ),
    }
}
