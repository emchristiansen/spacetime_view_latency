//! The frozen candidate version, recovered without widening the pilot's sealed type.

use crate::indexed_sender_view_calibration_pilot::candidate_version::INDEXED_SENDER_VIEW_CALIBRATION_VERSION;

/// The numeric candidate version every admissible record must carry.
///
/// **Recovered through serde rather than by an accessor or a restated literal**, and both refusals
/// are deliberate. `CandidateVersion` has a private field and no getter; adding one would widen a
/// sealed pilot type for this analyzer's convenience, which the §569 authorization forbids. Writing
/// `1` here instead would create a second copy of one truth, free to drift the moment the candidate
/// is revised — the exact duplication the pilot's own `WITH_CONFIRMED_READS` was corrected to remove.
///
/// Serializing the frozen constant is the same deliberate round trip the pilot documents as remaining
/// open, taken here for a stated purpose. It cannot silently disagree with the wire: it *is* the
/// value the writer emits, produced by the writer's own `Serialize` impl.
pub(crate) fn frozen_candidate_version() -> u32 {
    serde_json::to_string(&INDEXED_SENDER_VIEW_CALIBRATION_VERSION)
        .expect("a transparent u32 newtype always serializes")
        .parse()
        .expect("CandidateVersion is #[serde(transparent)] over a u32, so it renders as an integer")
}
