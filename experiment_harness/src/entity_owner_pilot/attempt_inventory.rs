//! The frozen, ordered inventory of every predeclared Pilot attempt.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::pilot_params::PILOT_BLOCKS_USIZE;
use crate::manifest::schedule_seed::ScheduleSeed;

/// How many attempts one candidate's single-axis Pilot predeclares: one Arm and one matched Control
/// in each of the five blocks.
///
/// The spec fixes both halves of this product — "Pilot: five randomized matched blocks" and "each
/// matched block contains arm and control attempts; counts must include both" — so the count is
/// derived from [`PILOT_BLOCKS_USIZE`] rather than written as a literal ten. The spec's larger
/// figure of forty Pilot slots is scoped to four-corner interaction candidates, which multiply this
/// same product by their four corners; it is not this single-axis candidate's count.
pub(crate) const PILOT_ATTEMPT_COUNT: usize = PILOT_BLOCKS_USIZE * 2;

/// The complete, frozen, execution-ordered list of every attempt this Pilot will make.
///
/// The spec requires the attempt inventory to be "freeze[n] before execution" and one terminal
/// record appended for "every planned attempt". This type is that freeze: it is built once, before
/// the first server is provisioned, and thereafter only read. Because the driver iterates *this*
/// list rather than generating attempts as it goes, an attempt that is never reached is still a
/// known member of the inventory and can be recorded
/// [`NotRun`](super::attempt_outcome::AttemptOutcome::NotRun) with a reason — which is what makes
/// "report generation fails on missing or duplicate planned identities" a checkable property.
///
/// The stored order *is* the randomized execution order, derived deterministically from the recorded
/// seed. Persisting the order rather than recomputing it at read time means the ledger shows the
/// order actually used even if the derivation is later changed.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AttemptInventory {
    order: Vec<AttemptKey>,
}

impl AttemptInventory {
    /// Freeze the inventory for one candidate's single-axis Pilot, in the deterministic randomized
    /// execution order derived from `seed`.
    ///
    /// The spec requires randomizing "the temporal order of all arm/regime/block products globally
    /// from the recorded seed", with the arm/control order within each block randomized as its own
    /// domain-separated derivation. Phase 2 fills this in by copying the historical schedule's
    /// seeded-permutation construction under this candidate's own
    /// [`PILOT_BLOCK_ORDER_DOMAIN`](super::pilot_params::PILOT_BLOCK_ORDER_DOMAIN) and
    /// [`PILOT_ARM_CONTROL_ORDER_DOMAIN`](super::pilot_params::PILOT_ARM_CONTROL_ORDER_DOMAIN)
    /// labels, so this candidate's ordering can never collide with the historical campaign's against
    /// the same seed.
    pub(crate) fn frozen(_seed: ScheduleSeed) -> Result<Self> {
        todo!("Phase 2: derive the seeded block and arm/control permutation")
    }

    /// Every predeclared attempt in execution order.
    pub(crate) fn attempts(&self) -> &[AttemptKey] {
        &self.order
    }

    /// How many attempts were predeclared — always [`PILOT_ATTEMPT_COUNT`] for a sealed inventory.
    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    /// Seal a built order, failing loud unless it is exactly the predeclared set: the right count,
    /// with no logical slot appearing twice.
    ///
    /// Duplicate-slot rejection is the half that matters. A permutation bug that emitted one block
    /// twice and another never would still produce ten attempts, so a bare length check would pass
    /// it and the campaign would silently measure the wrong design.
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
