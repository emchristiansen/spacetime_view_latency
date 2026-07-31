//! The frozen composition one attempt runs against.

use serde::Serialize;

use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::four_way_expectation::FourWayExpectation;
use crate::control_registry_discovery_screen::screen_params::REGISTRY_CONTROLS;

/// What an attempt's server was seeded to hold, which target it times, and what all four caches must
/// therefore contain.
///
/// Derived wholly from the attempt identity rather than passed alongside it, so the composition a
/// record reports and the composition its identity implies cannot disagree. That is the same reason
/// the identity carries no target field: one source, one meaning.
///
/// The expectation is the full four-way one even though only one target is timed, because the
/// per-attempt proof the freeze requires covers all four caches.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct ScreenComposition {
    target_tag: &'static str,
    history_rows: u64,
    registry_controls: u64,
    rows_per_control: u64,
    expected: FourWayExpectation,
}

impl ScreenComposition {
    /// The composition this identity denotes.
    pub(crate) fn of(key: AttemptKey) -> Self {
        let rung = key.rung();
        Self {
            target_tag: key.target().canonical_tag(),
            history_rows: rung.history_rows(),
            registry_controls: REGISTRY_CONTROLS,
            rows_per_control: rung.rows_per_control(),
            expected: FourWayExpectation::at(rung),
        }
    }

    /// The four-way expectation this attempt must validate after timing.
    pub(crate) fn expected(self) -> FourWayExpectation {
        self.expected
    }
}
