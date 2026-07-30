//! Why decoding the calibration ledger into DTOs did not complete.

use std::fmt;

/// A syntactic ingestion failure: one calibration ledger line did not decode into the closed wire
/// contract. This covers an unknown, duplicate, or missing field, an unknown variant tag, or a type
/// mismatch.
///
/// Deliberately a *syntactic* boundary error over already-loaded text, exactly as
/// [`IngestError`](crate::analysis::ingest::ingest_error::IngestError) is for the campaign: opening
/// and reading the ledger file is the caller's typed concern, and **admission** — whether a decoded
/// record is a complete replicate §569 can analyse — is the separate concern of
/// [`CompleteReplicate`](super::complete_replicate::CompleteReplicate), never an ingest error.
#[derive(Debug)]
pub(crate) enum CalibrationIngestError {
    /// The line at this 1-based position could not be decoded into the closed wire contract.
    MalformedLine {
        line_number: u64,
        diagnostic: String,
    },
}

impl CalibrationIngestError {
    /// A line that did not decode into the closed wire contract, tagged with its 1-based position.
    pub(crate) fn malformed_line(line_number: u64, diagnostic: String) -> Self {
        Self::MalformedLine {
            line_number,
            diagnostic,
        }
    }

    /// The 1-based position of the offending line in the ledger.
    pub(crate) fn line_number(&self) -> u64 {
        match self {
            CalibrationIngestError::MalformedLine { line_number, .. } => *line_number,
        }
    }

    /// The human-readable diagnostic for the failure.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            CalibrationIngestError::MalformedLine { diagnostic, .. } => diagnostic,
        }
    }
}

impl fmt::Display for CalibrationIngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalibrationIngestError::MalformedLine {
                line_number,
                diagnostic,
            } => write!(f, "malformed calibration ledger line {line_number}: {diagnostic}"),
        }
    }
}
