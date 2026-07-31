//! The frozen, ordered inventory of every predeclared Pilot attempt.

use anyhow::{ensure, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::candidate_id::CandidateId;
use crate::entity_owner_pilot::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::entity_owner_pilot::experiment_axis::ExperimentAxis;
use crate::entity_owner_pilot::pilot_block_index::PilotBlockIndex;
use crate::entity_owner_pilot::pilot_params::{
    PILOT_ARM_CONTROL_ORDER_DOMAIN, PILOT_BLOCKS_USIZE, PILOT_BLOCK_ORDER_DOMAIN,
};
use crate::entity_owner_pilot::retry_ordinal::RetryOrdinal;
use crate::entity_owner_pilot::stage_repetition::StageRepetition;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::run_role::RunRole;

/// Total ordering key for one Pilot block: its SHA-256 digest, then its block index as a
/// deterministic tiebreak, so the induced order is total even on a digest tie. Mirrors the
/// historical `BlockOrderKey` (`plan::schedule::schedule`); the block index alone is the whole
/// coordinate here because this Pilot walks a single candidate and axis.
type PilotBlockOrderKey = ([u8; 32], u32);

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
    ///
    /// Two independent domain-separated derivations, mirroring the historical schedule: the blocks
    /// are globally permuted ([`seeded_block_order`]), then each block's matched pair is emitted in
    /// its own randomized role order ([`ordered_roles`]). The pair stays adjacent, so a matched
    /// comparison is never split across the whole campaign's drift.
    pub(crate) fn frozen(seed: ScheduleSeed) -> Result<Self> {
        let mut order = Vec::with_capacity(PILOT_ATTEMPT_COUNT);
        for block in seeded_block_order(seed) {
            for role in ordered_roles(block, seed) {
                order.push(AttemptKey::new(
                    CandidateId::EntityOwnerSenderView,
                    ExperimentAxis::UnrelatedGlobalRows,
                    role,
                    StageRepetition::Pilot(block),
                    RetryOrdinal::ORIGINAL,
                    ENTITY_OWNER_SENDER_VIEW_VERSION,
                ));
            }
        }
        Self::sealed(order)
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

/// Every Pilot block in the deterministic randomized order derived from `seed`.
///
/// Copies [`Schedule::seeded_permutation`](crate::plan::schedule::Schedule): each block is assigned
/// a [`PilotBlockOrderKey`] and the vector is sorted by it. Sorting a `Vec` is a permutation, so the
/// result is exactly [`PilotBlockIndex::ALL`] reordered — no block added, dropped, or duplicated.
fn seeded_block_order(seed: ScheduleSeed) -> Vec<PilotBlockIndex> {
    let mut keyed: Vec<(PilotBlockOrderKey, PilotBlockIndex)> = PilotBlockIndex::ALL
        .into_iter()
        .map(|block| (block_order_key(block, seed), block))
        .collect();
    keyed.sort_by(|(left, _), (right, _)| left.cmp(right));
    keyed.into_iter().map(|(_, block)| block).collect()
}

/// The total ordering key for one block under `seed`.
///
/// The digest is taken over a domain-separated subject built from stable values — never `Debug`
/// output — so the derived order is reproducible against a recorded seed and cannot be moved by a
/// Rust rename.
fn block_order_key(block: PilotBlockIndex, seed: ScheduleSeed) -> PilotBlockOrderKey {
    let block_index = block.get();
    let subject = format!(
        "{};seed={};block={}",
        PILOT_BLOCK_ORDER_DOMAIN,
        seed.get(),
        block_index
    );
    let mut hasher = Sha256::new();
    hasher.update(subject.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    (digest, block_index)
}

/// One block's Arm and matched Control in their seed-randomized execution order, returned together
/// as one array so the matched pair's adjacency is structural rather than caller-maintained. The
/// pair is always exactly `{Arm, Control}`; only their order varies with the seed.
fn ordered_roles(block: PilotBlockIndex, seed: ScheduleSeed) -> [RunRole; 2] {
    if control_first(block, seed) {
        [RunRole::Control, RunRole::Arm]
    } else {
        [RunRole::Arm, RunRole::Control]
    }
}

/// Whether this block's matched Control executes before its Arm, derived under a domain distinct
/// from the block-order permutation so the role bit is its own derivation rather than a reuse of
/// that permutation's key. Copies [`BlockRun::ordered_runs`](crate::plan::schedule::BlockRun)'s
/// low-bit extraction.
fn control_first(block: PilotBlockIndex, seed: ScheduleSeed) -> bool {
    let subject = format!(
        "{};seed={};block={}",
        PILOT_ARM_CONTROL_ORDER_DOMAIN,
        seed.get(),
        block.get()
    );
    let mut hasher = Sha256::new();
    hasher.update(subject.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[0] & 1 == 1
}

#[cfg(test)]
mod tests;
