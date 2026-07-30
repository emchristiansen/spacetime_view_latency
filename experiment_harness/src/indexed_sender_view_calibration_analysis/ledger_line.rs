//! One decoded calibration ledger record, together with where in the file it came from.

use crate::indexed_sender_view_calibration_analysis::calibration_record_dto::CalibrationRecordDto;

/// A decoded record paired with its 1-based source line number.
///
/// **The position is minted once, at decode, from the file's own line order**, and travels with the
/// record from then on. An earlier shape derived it later from a vector index, which is only correct
/// while that vector is the whole file in original order — an invariant nothing enforced and every
/// intermediate step could quietly break. Recording it where the fact actually exists removes the
/// need for that invariant.
///
/// Fields are readable across the namespace, exactly as
/// [`RefusedLine`](super::refused_line::RefusedLine) exposes its own position and reason: this is a
/// position/payload pair whose halves are read together, not a type with behaviour to defend. What is
/// defended is the *ledger* — a [`CalibrationLedger`](super::calibration_ledger::CalibrationLedger)
/// cannot be assembled from values of this type, so being able to read a line grants no way to
/// manufacture a file.
#[derive(Debug, Clone)]
pub(crate) struct LedgerLine {
    /// Where this record sits in the ledger, counting from one.
    pub(crate) line_number: u64,
    /// The record as decoded under the closed wire contract.
    pub(crate) record: CalibrationRecordDto,
}
