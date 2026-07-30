//! Why the §569 analysis could not be produced.

use std::fmt;
use std::path::PathBuf;

use crate::indexed_sender_view_calibration_analysis::admission_refusal::AdmissionRefusal;
use crate::indexed_sender_view_calibration_analysis::calibration_ingest_error::CalibrationIngestError;
use crate::indexed_sender_view_calibration_analysis::pair_refusal::PairRefusal;

/// The typed reason no diagnostics report was produced.
///
/// Three genuinely different failures, kept apart because they call for different responses: the
/// artifact could not be read at all, its bytes are not this ledger's contract, or its records are
/// honest but do not constitute the pair §569 is stated over. Collapsing them would leave an operator
/// unable to tell a wrong `--ledger` path from a run that legitimately produced only one replicate.
#[derive(Debug)]
pub(crate) enum CalibrationAnalysisError {
    /// The ledger file could not be opened or read.
    ///
    /// Deliberately distinct from a malformed line, mirroring the campaign's separation of a
    /// report-input failure from an [`IngestError`](crate::analysis::ingest::ingest_error::IngestError):
    /// a path/open/read failure is never collapsed into "malformed".
    LedgerUnreadable { path: PathBuf, diagnostic: String },
    /// A line did not decode into the closed wire contract.
    Malformed(CalibrationIngestError),
    /// The ledger decoded, but its admitted records are not the frozen inventory's pair.
    ///
    /// **This is terminal for `W`.** §568 allows no hidden retry and the freeze contains exactly two
    /// originals, so an unpairable ledger does not mean "run it again" — it means the calibration
    /// coverage is lost and Control redesigns or defers rather than manufacturing a count.
    ///
    /// Carries **both** the pairing refusal and every per-record admission refusal with its 1-based
    /// line number, because those reasons diagnose *how* the coverage was lost. "Expected ordinals
    /// {0, 1}, found {0}" alone would leave a reader unable to see that line 2 was replicate 1
    /// refused for a short series — a different diagnosis, informing a different redesign, from that
    /// slot never having been reached at all.
    NotPairable {
        refusal: PairRefusal,
        refused: Vec<(u64, AdmissionRefusal)>,
    },
}

impl fmt::Display for CalibrationAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalibrationAnalysisError::LedgerUnreadable { path, diagnostic } => write!(
                f,
                "the calibration ledger at {} could not be read: {diagnostic}",
                path.display()
            ),
            CalibrationAnalysisError::Malformed(error) => write!(f, "{error}"),
            CalibrationAnalysisError::NotPairable { refusal, refused } => {
                write!(f, "{refusal}")?;
                for (line_number, reason) in refused {
                    write!(f, "\n  line {line_number}: {reason}")?;
                }
                Ok(())
            }
        }
    }
}
