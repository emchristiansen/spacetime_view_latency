//! The untrusted wire form of a sealed series' composition proof.

use serde::Deserialize;

/// The wire form of
/// [`VerifiedPopulation`](crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation).
///
/// **Proven against the writer, not guessed.** That type is a plain `#[derive(Serialize)]` struct
/// with three private `u64` fields named `arm_rows`, `witness_rows`, and `verified_appends`, and no
/// serde attribute of any kind, so it serializes as a three-key JSON object under exactly those
/// names.
///
/// The token discipline does **not** cross the wire: this is an ordinary deserializable struct, and
/// holding one proves nothing about composition. That is the whole reason admission is re-derived
/// here by [`CompleteReplicate`](super::complete_replicate::CompleteReplicate) rather than inherited
/// from the pilot's capability type.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerifiedPopulationDto {
    /// Rows the pilot proved present in the arm cache.
    pub(crate) arm_rows: u64,
    /// Rows the pilot proved present in the witness cache.
    pub(crate) witness_rows: u64,
    /// Appends the pilot proved landed in the subscriber's own slice.
    pub(crate) verified_appends: u64,
}
