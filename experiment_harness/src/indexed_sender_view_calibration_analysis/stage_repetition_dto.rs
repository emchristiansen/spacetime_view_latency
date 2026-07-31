//! The untrusted wire form of which replicate a record belongs to.

use serde::Deserialize;

/// The wire form of
/// [`StageRepetition`](crate::indexed_sender_view_calibration_pilot::stage_repetition::StageRepetition).
///
/// **The nesting is proven, not guessed.** `StageRepetition` is an attribute-free enum with the
/// single newtype variant `Calibration(CalibrationReplicate)`, and `CalibrationReplicate` is
/// `#[serde(transparent)]` over a `u32`. Serde's default external tagging therefore writes
/// `{"Calibration":0}` — a tagged integer, not a nested object.
///
/// This is the analyzer's only source of a replicate ordinal, and the ordinal is what makes
/// [`ReplicatePair`](super::replicate_pair::ReplicatePair)'s distinctness requirement checkable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum StageRepetitionDto {
    /// The 0-based replicate index this record belongs to.
    Calibration(u32),
}

impl StageRepetitionDto {
    /// The 0-based replicate index.
    pub(crate) fn replicate(self) -> u32 {
        match self {
            StageRepetitionDto::Calibration(replicate) => replicate,
        }
    }
}
