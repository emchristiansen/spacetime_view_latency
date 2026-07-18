//! Why decoding the untrusted NDJSON artifact into DTOs did not complete.

use std::fmt;

/// A syntactic ingestion failure: one NDJSON line did not decode into the closed wire contract. This
/// covers an unknown or duplicate field, a missing field, an unknown variant tag, a type mismatch, or
/// a body shape that disagrees with its record kind. It is deliberately a *syntactic* boundary error
/// over already-loaded text: opening and reading the artifact file is owned by a higher typed
/// report-input error (a path/open/read failure is never collapsed into a `MalformedLine`), and
/// completeness, cross-reference, and recomputation failures are the separate typed integrity outcome
/// of the `validate` pass ([`IntegrityError`](crate::analysis::validate::integrity_error::IntegrityError)),
/// never an [`IngestError`].
#[derive(Debug)]
pub(crate) enum IngestError {
    /// The line at this 1-based position could not be decoded into the closed wire contract.
    MalformedLine {
        line_number: u64,
        diagnostic: String,
    },
}

impl IngestError {
    /// A line that did not decode into the closed wire contract, tagged with its 1-based position.
    pub(crate) fn malformed_line(line_number: u64, diagnostic: String) -> Self {
        Self::MalformedLine {
            line_number,
            diagnostic,
        }
    }

    /// The 1-based position of the offending line in the artifact.
    pub(crate) fn line_number(&self) -> u64 {
        match self {
            IngestError::MalformedLine { line_number, .. } => *line_number,
        }
    }

    /// The human-readable diagnostic for the failure.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            IngestError::MalformedLine { diagnostic, .. } => diagnostic,
        }
    }
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestError::MalformedLine {
                line_number,
                diagnostic,
            } => write!(f, "malformed NDJSON line {line_number}: {diagnostic}"),
        }
    }
}
