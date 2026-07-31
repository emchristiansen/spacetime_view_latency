//! The untrusted wire form of which candidate an attempt exercised.

use serde::Deserialize;

/// The wire form of
/// [`CandidateId`](crate::indexed_sender_view_calibration_pilot::candidate_id::CandidateId).
///
/// **Mirrored and matched, never skipped.** That the source enum has exactly one variant constrains
/// what the *pilot writes*; it constrains nothing about the bytes in a file this analyzer is pointed
/// at. Declining to decode this component would admit a line naming any candidate at all — including
/// one whose series was measured against different module code — because a field that is never
/// deserialized is never checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum CandidateIdDto {
    /// The indexed `control_activity` sender view.
    IndexedControlActivitySenderView,
}
