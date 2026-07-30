//! How an attempt that actually ran ended.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::attempt_failure::AttemptFailure;
use crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries;

/// The terminal outcome of an attempt that provisioned and ran: it either recorded a complete
/// calibration series or failed.
///
/// **There is deliberately no `Complete` variant.** The discovery screen's outcome is `Complete
/// { evidence }` because a completed E3 attempt yields evidence for a candidate. This pilot yields
/// nothing of the kind: its only admissible output is material for choosing a sample count, so its
/// success shape is named for what it is. A ledger line from this pilot cannot spell a finished
/// measurement as a finished *result*, because there is no variant with that meaning to write.
///
/// Deliberately excludes "not run". A slot that never ran has no provision provenance and no host
/// observations, so it cannot inhabit the same shape as one that did — see
/// [`CalibrationRecord`](super::calibration_record::CalibrationRecord), where the two are separate
/// variants rather than one variant with optional fields.
///
/// **No `Debug`**, inherited from both members' retained samples.
#[derive(Clone, Serialize)]
pub(crate) enum AttemptedOutcome {
    /// The attempt ran its paced batch to the frozen count and every row in both caches validated.
    ///
    /// The series inside is complete by construction and was sealed against a verifier-minted
    /// population token, so this variant cannot be reached by a short run or by a same-cardinality
    /// substitution.
    CalibrationRecorded { series: CalibrationSeries },
    /// The attempt ran and did not record a complete, validated series.
    Failed { failure: AttemptFailure },
}
