//! Decode a whole calibration ledger into the untrusted wire DTOs, one line at a time.

use crate::indexed_sender_view_calibration_analysis::calibration_ingest_error::CalibrationIngestError;
use crate::indexed_sender_view_calibration_analysis::calibration_record_dto::CalibrationRecordDto;

/// Decode every line of an already-loaded calibration ledger into a [`CalibrationRecordDto`],
/// preserving file order. Each line is decoded independently under the closed wire contract, and the
/// first line that fails aborts with a typed error tagged with its 1-based position.
///
/// Copies [`parse_ndjson`](crate::analysis::ingest::parse_ndjson::parse_ndjson) exactly, including
/// its boundary: this is purely the *syntactic* pass over in-memory text. Reading the file is the
/// caller's concern, and admission — which decoded records are complete replicates — is the separate
/// concern of [`CompleteReplicate`](super::complete_replicate::CompleteReplicate).
///
/// Blank lines are not skipped. A ledger written by the pilot's `append_all` has exactly one record
/// per line with no blank separators, so a blank line means the artifact is not the one this analyzer
/// was pointed at, and silently tolerating it would let a truncated or concatenated file look whole.
pub(crate) fn parse_calibration_ndjson(
    contents: &str,
) -> std::result::Result<Vec<CalibrationRecordDto>, CalibrationIngestError> {
    todo!("per-line decode with 1-based position tagging")
}

#[cfg(test)]
mod tests;
