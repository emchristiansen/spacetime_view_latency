//! What all four caches must hold at one rung.

use serde::Serialize;

use crate::control_registry_discovery_screen::screen_params::REGISTRY_CONTROLS;
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;

/// The frozen four-way expectation every attempt validates after timing.
///
/// Four counts, not one. The timed target's cardinality alone cannot establish composition: an
/// attempt that timed Arm A and saw K rows has shown nothing about whether the history it was
/// supposed to be sitting on top of was actually seeded. Three of the four are `K` at both rungs and
/// only the comparator's Control tracks `N` — that invariance *is* the composition claim, so all
/// four are checked or none of it is proven.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct FourWayExpectation {
    arm_a: u64,
    control_a: u64,
    arm_b: u64,
    control_b: u64,
}

impl FourWayExpectation {
    /// The expectation at one rung, derived rather than supplied so it cannot be stated
    /// inconsistently with the rung an attempt actually ran at.
    pub(crate) fn at(rung: ScreenRung) -> Self {
        Self {
            arm_a: REGISTRY_CONTROLS,
            control_a: REGISTRY_CONTROLS,
            arm_b: REGISTRY_CONTROLS,
            control_b: rung.history_rows(),
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
