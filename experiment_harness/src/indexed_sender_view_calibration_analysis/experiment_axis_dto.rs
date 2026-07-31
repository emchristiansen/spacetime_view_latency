//! The untrusted wire form of which axis an attempt swept.

use serde::Deserialize;

/// The wire form of
/// [`ExperimentAxis`](crate::indexed_sender_view_calibration_pilot::experiment_axis::ExperimentAxis).
///
/// Mirrored and matched for the same reason as
/// [`CandidateIdDto`](super::candidate_id_dto::CandidateIdDto): an undecoded field is an unchecked
/// one, whatever the writing enum's shape happens to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum ExperimentAxisDto {
    /// Unrelated rows held by a non-connecting owner.
    UnrelatedGlobalRows,
}
