//! One attempt's sealed cold-apply evidence.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::control_registry_discovery_screen::four_way_composition::FourWayComposition;

/// The single cold subscription apply duration an attempt yields, sealed only if it is usable *and*
/// all four caches matched.
///
/// The raw nanosecond duration is retained as the sample itself, not a summary of it: at
/// `sample_count = 1` a mean, median, and raw value coincide, and recording it as a summary would
/// invite a later reader to believe a distribution was estimated. The frozen method facts —
/// `sample_count`, `with_confirmed_reads`, and the channel — are carried by
/// [`MethodFacts`](super::method_facts::MethodFacts) on *every* record, so a ledger-only check never
/// has to special-case an outcome to find them.
///
/// This is the only type that can hold a duration, and it cannot be built from an invalid one or
/// from an unproven composition — the single gate through which a measurement becomes evidence.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct ColdApplyEvidence {
    apply_nanos: u128,
    composition: FourWayComposition,
}

impl ColdApplyEvidence {
    /// Seal one attempt's cold apply, failing loud unless the statistic is usable and every one of
    /// the four caches matched its frozen expectation.
    ///
    /// A missing, non-finite, or nonpositive cell statistic invalidates its cell rather than
    /// counting as flat, so a zero duration is rejected here rather than written and reasoned about
    /// later. The four-way check is in the same constructor because an apply duration measured
    /// against a composition that was never established is not a cheap reading of the right one — it
    /// is not evidence at all. Timing the right cache while the other three were mis-seeded is
    /// exactly the failure this rejects.
    pub(crate) fn sealed(apply_nanos: u128, composition: FourWayComposition) -> Result<Self> {
        ensure!(
            apply_nanos > 0,
            "a cold apply duration must be strictly positive, got {apply_nanos} ns"
        );
        ensure!(
            composition.matches(),
            "the four-way composition check failed, so this pair is invalid: {}",
            composition.mismatches().join("; "),
        );
        Ok(Self {
            apply_nanos,
            composition,
        })
    }
}
