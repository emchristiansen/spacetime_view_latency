//! No two representable attempt identities share one artifact directory name.

use std::collections::HashMap;

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

/// Coverage: two identities that produced the same tag would name one directory, and the second
/// attempt's exclusive creation would fail — reading as a repeated identity when it is in fact an
/// incomplete spelling. The block coordinate is the specific hazard the spec calls out: the Pilot's
/// five matched blocks differ *only* by block index at the same candidate, axis, rung, role,
/// version, and retry, so a tag omitting it would collide every matched pair. The retry ordinal is
/// the same hazard for a slot's one permitted retry, whose whole purpose is to never overwrite its
/// predecessor's evidence.
///
/// Rather than checking those two coordinates individually, this enumerates the entire space of
/// representable identities — every rung of the frozen ladder, both roles, all five blocks, both
/// retry ordinals — and requires the tags to be pairwise distinct. Injectivity over the whole space
/// is the property the artifact layout actually needs, and it subsumes any single-coordinate check;
/// a coordinate dropped from the spelling shows up here as the exact pair it collided.
///
/// The remaining components admit one value each today, so they cannot be varied — that is why the
/// sibling frozen-spelling test pins their tokens.
#[test]
fn every_coordinate_changes_the_canonical_tag() {
    let mut by_tag: HashMap<String, AttemptKey> = HashMap::new();

    for scale in ScalePoint::ladder(ExperimentAxis::UnrelatedGlobalRows) {
        for role in [RunRole::Arm, RunRole::Control] {
            for block in PilotBlockIndex::ALL {
                for retry in [RetryOrdinal::ORIGINAL, RetryOrdinal::RETRY] {
                    let attempt = AttemptKey::new(
                        CandidateId::EntityOwnerSenderView,
                        scale,
                        role,
                        StageRepetition::Pilot(block),
                        retry,
                        ENTITY_OWNER_SENDER_VIEW_VERSION,
                    );
                    let tag = attempt.canonical_tag();
                    if let Some(previous) = by_tag.insert(tag.clone(), attempt) {
                        panic!(
                            "two distinct attempt identities share the artifact directory name \
                             {tag:?}: {previous:?} and {attempt:?}"
                        );
                    }
                }
            }
        }
    }

    assert_eq!(
        by_tag.len(),
        6 * 2 * 5 * 2,
        "the enumeration must cover every rung, role, block, and retry ordinal in existence"
    );
}
