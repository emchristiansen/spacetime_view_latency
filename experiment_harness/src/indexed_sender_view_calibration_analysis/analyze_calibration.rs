//! Read one calibration ledger and produce its §569 diagnostics.

use std::path::Path;

use crate::indexed_sender_view_calibration_analysis::calibration_analysis_error::CalibrationAnalysisError;
use crate::indexed_sender_view_calibration_analysis::calibration_diagnostics_report::CalibrationDiagnosticsReport;
use crate::indexed_sender_view_calibration_analysis::ledger_admission::LedgerAdmission;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;

/// The whole read-only §569 path: load the ledger, decode every line, admit what qualifies, pair the
/// frozen inventory's two replicates, and compute the diagnostics.
///
/// **Provisions nothing, measures nothing, mutates nothing.** It opens one file for reading and
/// writes only to stdout through its caller. It cannot start a server, run the host waiter, or touch
/// an evidence directory — there is no code path here that does any of those, which is what makes it
/// safe to run while the pilot's own execution hold is in force.
///
/// The four stages stay separate and typed, exactly as the campaign's `analyze` keeps ingest,
/// validate, and report apart: a read failure, a malformed line, and an unpairable set of honest
/// records are three different outcomes and are reported as three different things.
pub(crate) fn analyze_calibration(
    ledger: &Path,
) -> Result<CalibrationDiagnosticsReport, CalibrationAnalysisError> {
    let contents = std::fs::read_to_string(ledger).map_err(|error| {
        CalibrationAnalysisError::LedgerUnreadable {
            path: ledger.to_path_buf(),
            diagnostic: error.to_string(),
        }
    })?;
    let records =
        parse_calibration_ndjson(&contents).map_err(CalibrationAnalysisError::Malformed)?;

    // The whole ledger is offered for admission and the outcome travels onward intact: `LedgerAdmission`
    // holds the refusals alongside the admissions, so pairing sees both and this function has no way to
    // forward one without the other.
    let admission = LedgerAdmission::of(&records);
    let pair = ReplicatePair::of(admission).map_err(CalibrationAnalysisError::NotPairable)?;
    Ok(CalibrationDiagnosticsReport::of(&pair))
}

#[cfg(test)]
mod tests;
