//! The canonical identity spelling is exactly the frozen component, byte for byte.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

/// Coverage: this string is the directory name an attempt's retained evidence lives under, so it is
/// part of the preregistration rather than an implementation detail — a reader who was told where a
/// finding's artifacts are must still find them after any later refactor. The spec freezes the
/// component exactly.
///
/// The expected value is therefore written as a *literal*, deliberately, and not rebuilt from the
/// canonical accessors it is assembled from. Rebuilding it would restate the implementation and
/// pass under any coordinated change to both; a literal is the only form that fails when the frozen
/// spelling moves. That is also what makes this the test that catches a `Debug`, variant-name, or
/// serde spelling creeping into any one of the six components — each such change alters these bytes.
#[test]
fn the_canonical_tag_is_the_frozen_spelling() {
    let scale = ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 3)
        .expect("the frozen unrelated-global-rows ladder has a fourth rung");
    let attempt = AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        scale,
        RunRole::Control,
        StageRepetition::Pilot(PilotBlockIndex::ALL[2]),
        RetryOrdinal::RETRY,
        ENTITY_OWNER_SENDER_VIEW_VERSION,
    );

    assert_eq!(
        attempt.canonical_tag(),
        "candidate=entity-owner-sender-view;axis=unrelated-global-rows;rung=3;role=control;\
         stage=pilot;block=2;version=1;retry=1",
        "the artifact directory component is frozen by the spec; changing it moves evidence a \
         recorded finding already points at"
    );
}
