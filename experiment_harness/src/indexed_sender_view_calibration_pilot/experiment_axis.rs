//! The axis this pilot holds still.

use serde::Serialize;

/// The axis the screen this pilot calibrates will sweep, recorded here at its single baseline
/// position.
///
/// Named even though nothing varies, because attempt identity binds a scale point and a series
/// recorded at the baseline rung must not later be mistaken for one recorded elsewhere on the
/// ladder. The pilot moves no axis at all: it is method calibration, and a moving axis is exactly
/// what would turn its two replicates into the comparison the ceiling forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ExperimentAxis {
    UnrelatedGlobalRows,
}
