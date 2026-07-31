//! Read one calibration ledger and produce its §569 diagnostics.

use std::path::Path;

use crate::indexed_sender_view_calibration_analysis::calibration_analysis_error::CalibrationAnalysisError;
use crate::indexed_sender_view_calibration_analysis::calibration_diagnostics_report::CalibrationDiagnosticsReport;
use crate::indexed_sender_view_calibration_analysis::calibration_ledger::CalibrationLedger;
use crate::indexed_sender_view_calibration_analysis::ledger_admission::LedgerAdmission;
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
    // Reading and decoding are one step, inside the ledger type: this function never holds the
    // contents, so it has no opportunity to offer a slice of them for admission.
    let ledger = CalibrationLedger::read(ledger)?;

    // The whole file is offered for admission, and the outcome cannot be split: `LedgerAdmission`'s
    // only exit yields the complete replicates or the refusal, so this function has no way to forward
    // admissions while leaving refusals behind.
    let pair = ReplicatePair::of(LedgerAdmission::of(ledger))
        .map_err(CalibrationAnalysisError::NotPairable)?;
    Ok(CalibrationDiagnosticsReport::of(&pair))
}

#[cfg(test)]
mod tests;
