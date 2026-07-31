//! Which of the independent calibration attempts a record belongs to.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::calibration_params::CALIBRATION_ATTEMPTS_USIZE;

/// A validated replicate index. [`Self::ALL`] is the only source of values, so a replicate outside
/// the frozen count is unrepresentable rather than merely unwritten.
///
/// **Replicates, not blocks.** A block index in the discovery screen selects a counterbalanced rung
/// order; there is nothing to counterbalance here, because the pilot runs one rung and one target.
/// The index exists solely to keep the two attempts' identities distinct so their series are
/// recorded and analysed separately — which is what makes between-attempt disagreement, one of the
/// four things the spec's decision rule weighs, visible at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct CalibrationReplicate(u32);

impl CalibrationReplicate {
    /// Every valid replicate index in ascending order — the only values in existence.
    pub(crate) const ALL: [CalibrationReplicate; CALIBRATION_ATTEMPTS_USIZE] = {
        let mut all = [CalibrationReplicate(0); CALIBRATION_ATTEMPTS_USIZE];
        let mut i = 0usize;
        while i < CALIBRATION_ATTEMPTS_USIZE {
            all[i] = CalibrationReplicate(i as u32);
            i += 1;
        }
        all
    };

    /// The 0-based replicate number.
    pub(crate) fn get(self) -> u32 {
        self.0
    }
}
