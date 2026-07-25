//! The frozen, ordered inventory of every predeclared Pilot attempt.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::pilot_params::PILOT_BLOCKS_USIZE;
use crate::manifest::schedule_seed::ScheduleSeed;

/// How many attempts this single-axis Pilot predeclares: one Arm and one matched Control per block.
///
/// Derived from [`PILOT_BLOCKS_USIZE`] rather than written as a literal ten. The spec's figure of
/// forty Pilot slots is scoped to four-corner interaction candidates, which multiply this same
/// product by their four corners; it is not this candidate's count.
pub(crate) const PILOT_ATTEMPT_COUNT: usize = PILOT_BLOCKS_USIZE * 2;

/// The complete, frozen, execution-ordered list of every attempt this Pilot will make.
///
/// Built once before the first server is provisioned, then only read. Because the driver iterates
/// *this* list rather than generating attempts as it goes, an attempt that is never reached is still
/// a known member and can be recorded
/// [`NotRun`](super::attempt_outcome::AttemptOutcome::NotRun) — which is what makes "report
/// generation fails on missing or duplicate planned identities" checkable.
///
/// The stored order *is* the randomized execution order. Persisting it rather than recomputing at
/// read time means the ledger shows the order actually used even if the derivation later changes.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AttemptInventory {
    order: Vec<AttemptKey>,
}

impl AttemptInventory {
    /// Freeze the inventory in the deterministic randomized execution order derived from `seed`.
    pub(crate) fn frozen(_seed: ScheduleSeed) -> Result<Self> {
        todo!("Phase 2: derive the seeded block and arm/control permutation")
    }

    /// Every predeclared attempt in execution order.
    pub(crate) fn attempts(&self) -> &[AttemptKey] {
        &self.order
    }

    /// Seal a built order, failing loud unless it is the right count with no logical slot repeated.
    ///
    /// Duplicate-slot rejection is the half that matters: a permutation bug emitting one block twice
    /// and another never would still produce ten attempts, so a bare length check would pass it and
    /// the campaign would silently measure the wrong design.
    fn sealed(order: Vec<AttemptKey>) -> Result<Self> {
        ensure!(
            order.len() == PILOT_ATTEMPT_COUNT,
            "a frozen Pilot inventory must predeclare exactly {PILOT_ATTEMPT_COUNT} attempts, \
             got {}",
            order.len(),
        );
        for (position, attempt) in order.iter().enumerate() {
            for (other_position, other) in order.iter().enumerate().skip(position + 1) {
                ensure!(
                    !attempt.same_logical_slot(*other),
                    "the frozen Pilot inventory repeats a logical slot at positions {position} and \
                     {other_position}; every predeclared attempt must address a distinct slot",
                );
            }
        }
        Ok(Self { order })
    }
}
