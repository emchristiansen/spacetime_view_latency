//! The frozen, ordered inventory of every predeclared calibration attempt.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::attempt_key::AttemptKey;
use crate::indexed_sender_view_calibration_pilot::calibration_params::CALIBRATION_ATTEMPTS_USIZE;
use crate::indexed_sender_view_calibration_pilot::calibration_replicate::CalibrationReplicate;

/// How many attempts this pilot predeclares: one per replicate.
///
/// Derived from the frozen replicate count rather than written as a literal two, so a change to it
/// cannot leave a stale count behind that the seal would then enforce. There is no rung or target
/// factor in this product because both are singletons of the freeze, fixed inside
/// [`AttemptKey::calibration`] rather than iterated here.
pub(crate) const CALIBRATION_ATTEMPT_COUNT: usize = CALIBRATION_ATTEMPTS_USIZE;

/// The complete, frozen, execution-ordered list of every attempt this pilot will make.
///
/// Built once before the first server is provisioned, then only read. Because the driver iterates
/// *this* list rather than generating attempts as it goes, an attempt that is never reached is still
/// a known member and can be recorded
/// [`NotRun`](super::not_run_reason::NotRunReason) — which is what makes the spec's requirement that
/// both replicates be accounted for checkable at all.
///
/// **No seeded permutation, deliberately.** The discovery screen permutes targets within a group to
/// avoid a fixed position effect; there is one target and one rung here, so there is nothing to
/// permute and a seeded order would be ceremony. The two replicates run in ascending order, which is
/// also the only order two independent attempts can have.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AttemptInventory {
    order: Vec<AttemptKey>,
}

impl AttemptInventory {
    /// Freeze the inventory in execution order.
    pub(crate) fn frozen() -> Result<Self> {
        let order = CalibrationReplicate::ALL
            .into_iter()
            .map(AttemptKey::calibration)
            .collect();
        Self::sealed(order)
    }

    /// Every predeclared attempt in execution order.
    pub(crate) fn attempts(&self) -> &[AttemptKey] {
        &self.order
    }

    /// Seal a built order, failing loud unless it is the right count with no logical slot repeated.
    ///
    /// Duplicate-slot rejection is the half that matters: a construction bug emitting one replicate
    /// twice and the other never would still produce two attempts, so a bare length check would pass
    /// it and the pilot would record one replicate under two identities — which is precisely the
    /// between-attempt disagreement the decision rule is meant to read.
    fn sealed(order: Vec<AttemptKey>) -> Result<Self> {
        ensure!(
            order.len() == CALIBRATION_ATTEMPT_COUNT,
            "a frozen calibration inventory must predeclare exactly {CALIBRATION_ATTEMPT_COUNT} \
             attempts, got {}",
            order.len(),
        );
        for (position, attempt) in order.iter().enumerate() {
            for (other_position, other) in order.iter().enumerate().skip(position + 1) {
                ensure!(
                    !attempt.same_logical_slot(*other),
                    "the frozen calibration inventory repeats a logical slot at positions \
                     {position} and {other_position}; every predeclared attempt must address a \
                     distinct slot",
                );
            }
        }
        Ok(Self { order })
    }
}

#[cfg(test)]
mod tests;
