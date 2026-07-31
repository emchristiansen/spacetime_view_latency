//! The untrusted wire form of which endpoint an attempt ran at.

use serde::Deserialize;

/// The wire form of
/// [`CalibrationRung`](crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung).
///
/// Mirrored and matched. The rung determines the unrelated population size the composition proof is
/// checked against, so accepting an unvalidated rung would let a series measured at some other
/// endpoint be admitted and then compared, per `W`, against one measured at the baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum CalibrationRungDto {
    /// The 1,000-row baseline global rung.
    Baseline,
}
