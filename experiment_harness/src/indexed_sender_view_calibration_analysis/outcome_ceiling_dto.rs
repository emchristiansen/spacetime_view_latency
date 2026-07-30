//! The untrusted wire form of the ceiling a record was produced under.

use serde::Deserialize;

/// The wire form of
/// [`OutcomeCeiling`](crate::indexed_sender_view_calibration_pilot::outcome_ceiling::OutcomeCeiling).
///
/// One variant, matching the source exactly. A ledger line written under a *wider* ceiling than the
/// one §568 authorized therefore fails to decode at all, which is the strongest refusal available at
/// this boundary: this analyzer cannot be pointed at evidence whose own record says it was permitted
/// to conclude more than calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum OutcomeCeilingDto {
    /// Evidence for choosing a within-cell sample count, and nothing further.
    MethodCalibrationOnly,
}
