//! The widest conclusion any record from this pilot may support.

use serde::Serialize;

/// The ceiling on what this pilot's evidence can be read as.
///
/// **One variant, written from a frozen constant onto every record.** The spec's decision authorizes
/// this pilot to produce evidence for freezing `W` and *nothing else*: no performance conclusion, no
/// scaling conclusion, no candidate outcome, no site disposition, no Arm/Control ratio. Carrying that
/// as a single-variant enum means no attempt can report having run under a wider ceiling, and a
/// later reader holding only the ledger sees the limit stated on the line rather than having to know
/// it from the spec.
///
/// A second variant added later would be a different authorization, and would have to be granted
/// before it could be written — which is the property this type is for. It is not a comment that a
/// future edit can quietly contradict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum OutcomeCeiling {
    /// Evidence for choosing a within-cell sample count, and nothing further.
    MethodCalibrationOnly,
}

/// The ceiling every record of this pilot carries.
pub(crate) const CALIBRATION_ONLY: OutcomeCeiling = OutcomeCeiling::MethodCalibrationOnly;
