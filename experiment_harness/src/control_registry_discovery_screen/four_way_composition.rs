//! The frozen four-way expectation paired with what was observed.

use serde::Serialize;

use crate::control_registry_discovery_screen::four_way_expectation::FourWayExpectation;
use crate::control_registry_discovery_screen::four_way_observation::FourWayObservation;

/// One attempt's composition check: the frozen expectation, the four observed cardinalities, and
/// whether every one of them matched.
///
/// Kept together in one value so an observation can never be read without the expectation it must be
/// judged against. The four caches are validated *after* the timed interval — validation is not part
/// of the estimand — but before any evidence is sealed, so a mismatch invalidates the pair rather
/// than being noticed afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct FourWayComposition {
    expected: FourWayExpectation,
    observed: FourWayObservation,
}

impl FourWayComposition {
    /// Pair an expectation with what was observed.
    pub(crate) fn checked(expected: FourWayExpectation, observed: FourWayObservation) -> Self {
        Self { expected, observed }
    }

    /// Whether all four caches matched exactly.
    ///
    /// Exact equality on every cache, with no tolerance anywhere: these are cardinalities of seeded
    /// relations, so "close" is simply wrong.
    pub(crate) fn matches(self) -> bool {
        self.mismatches().is_empty()
    }

    /// Every cache whose observed cardinality differed, named with both values.
    ///
    /// Returned as the full list rather than the first divergence, because which *combination* of
    /// caches is wrong distinguishes a seeding shortfall from a subscription applying against the
    /// wrong relation.
    pub(crate) fn mismatches(self) -> Vec<String> {
        let expected = self.expected;
        let observed = self.observed;
        [
            ("arm-a", expected.arm_a(), observed.arm_a()),
            ("control-a", expected.control_a(), observed.control_a()),
            ("arm-b", expected.arm_b(), observed.arm_b()),
            ("control-b", expected.control_b(), observed.control_b()),
        ]
        .into_iter()
        .filter(|(_, expected, observed)| expected != observed)
        .map(|(cache, expected, observed)| {
            format!("{cache}: expected {expected}, observed {observed}")
        })
        .collect()
    }
}
