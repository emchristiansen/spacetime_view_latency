//! Which stage and repetition an attempt belongs to.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::calibration_replicate::CalibrationReplicate;

/// The stage an attempt belongs to, carrying its repetition coordinate.
///
/// One variant, because this module runs one stage. It is an enum rather than a bare replicate index
/// so that the screen this pilot calibrates — a different stage entirely, with blocks and rungs —
/// adds its own vocabulary instead of reinterpreting calibration replicates as something they are
/// not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum StageRepetition {
    Calibration(CalibrationReplicate),
}
