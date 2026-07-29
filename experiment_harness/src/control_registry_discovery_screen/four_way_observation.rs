//! What all four caches actually held.

use serde::Serialize;

/// The four cache cardinalities read after the timed interval.
///
/// Recorded verbatim, including when they are wrong: a mismatch is the diagnostic content of a
/// semantic failure, so these are retained rather than collapsed to a pass/fail bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct FourWayObservation {
    arm_a: u64,
    control_a: u64,
    arm_b: u64,
    control_b: u64,
}

impl FourWayObservation {
    /// Record what the four caches held.
    pub(crate) fn read(arm_a: u64, control_a: u64, arm_b: u64, control_b: u64) -> Self {
        Self {
            arm_a,
            control_a,
            arm_b,
            control_b,
        }
    }

    pub(crate) fn arm_a(self) -> u64 {
        self.arm_a
    }

    pub(crate) fn control_a(self) -> u64 {
        self.control_a
    }

    pub(crate) fn arm_b(self) -> u64 {
        self.arm_b
    }

    pub(crate) fn control_b(self) -> u64 {
        self.control_b
    }
}
