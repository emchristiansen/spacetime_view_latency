//! The one permitted retry is the same identity with only its ordinal advanced, and it has no
//! successor of its own.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

/// Coverage: that the derivation moves the ordinal and *only* the ordinal. The assertion is equality
/// against an independently built key at `RETRY`, rather than merely that the ordinal advanced, so a
/// derivation that also altered another coordinate fails here instead of filing the retry's evidence
/// under a slot it never measured.
///
/// **What this does not claim to catch**, because the type system already does: a transposed pair of
/// arguments, since [`AttemptKey::new`] takes six distinct types, or a coordinate dropped as a
/// seventh is added, since that changes its arity. The hazards the method exists to remove are that
/// reconstruction would require exposing the private stage coordinate, and that a call site naming
/// values can name a well-typed wrong one. Neither is what this test observes; it observes the
/// method's own result.
///
/// The whole representable space is enumerated — every rung, both roles, all five blocks — rather
/// than one fixed identity, so the equality is checked where the coordinates genuinely differ rather
/// than at a single point where several happen to coincide.
///
/// It also pins the cap and its direction: the retry has no successor, so a slot cannot acquire a
/// third attempt, and the derived identity is never equal to the original it came from, which is
/// what stops a retry from overwriting its predecessor's evidence.
#[test]
fn the_one_permitted_retry_advances_only_the_ordinal() {
    let mut originals = 0;

    for scale in ScalePoint::ladder(ExperimentAxis::UnrelatedGlobalRows) {
        for role in [RunRole::Arm, RunRole::Control] {
            for block in PilotBlockIndex::ALL {
                let stage = StageRepetition::Pilot(block);
                let original = AttemptKey::new(
                    CandidateId::EntityOwnerSenderView,
                    scale,
                    role,
                    stage,
                    RetryOrdinal::ORIGINAL,
                    ENTITY_OWNER_SENDER_VIEW_VERSION,
                );
                let expected = AttemptKey::new(
                    CandidateId::EntityOwnerSenderView,
                    scale,
                    role,
                    stage,
                    RetryOrdinal::RETRY,
                    ENTITY_OWNER_SENDER_VIEW_VERSION,
                );

                let retry = original.next_retry().expect(
                    "an identity at the original ordinal always has its one permitted retry",
                );

                assert_eq!(
                    retry,
                    expected,
                    "the retry of {} must differ from it in the retry ordinal and nothing else",
                    original.canonical_tag(),
                );
                assert!(
                    original.same_logical_slot(retry),
                    "the retry of {} must address the same logical slot",
                    original.canonical_tag(),
                );
                assert_ne!(
                    original,
                    retry,
                    "the retry of {} must be a distinct durable identity, or it would overwrite \
                     its predecessor's evidence",
                    original.canonical_tag(),
                );
                assert_eq!(
                    retry.next_retry(),
                    None,
                    "the retry of {} must have no successor: the protocol permits one retry per \
                     slot",
                    original.canonical_tag(),
                );

                originals += 1;
            }
        }
    }

    assert_eq!(
        originals,
        6 * 2 * 5,
        "the enumeration must cover every rung, role, and block in existence"
    );
}
