//! The frozen, ordered inventory of every predeclared screen attempt.

use anyhow::{ensure, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::candidate_version::CONTROL_REGISTRY_DISCOVERY_VERSION;
use crate::control_registry_discovery_screen::experiment_axis::ExperimentAxis;
use crate::control_registry_discovery_screen::retry_ordinal::RetryOrdinal;
use crate::control_registry_discovery_screen::screen_block_index::ScreenBlockIndex;
use crate::control_registry_discovery_screen::screen_params::{
    SCREEN_BLOCKS_USIZE, SCREEN_TARGET_ORDER_DOMAIN,
};
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::control_registry_discovery_screen::screen_target::ScreenTarget;
use crate::control_registry_discovery_screen::stage_repetition::StageRepetition;
use crate::manifest::schedule_seed::ScheduleSeed;

/// Total ordering key for one target within a `(block, rung)` group: its SHA-256 digest, then its
/// canonical tag as a deterministic tiebreak, so the induced order is total even on a digest tie.
/// Copies the visible-rows probe's `ordered_attempts` key.
type ScreenTargetOrderKey = ([u8; 32], &'static str);

/// How many attempts this screen predeclares: four targets, at each of two rungs, in each of two
/// blocks.
///
/// Derived from the three frozen dimensions rather than written as a literal sixteen, so a change to
/// any dimension cannot leave a stale count behind that the seal would then enforce.
pub(crate) const SCREEN_ATTEMPT_COUNT: usize =
    SCREEN_BLOCKS_USIZE * ScreenRung::ALL.len() * ScreenTarget::ALL.len();

/// The complete, frozen, execution-ordered list of every attempt this screen will make.
///
/// Built once before the first server is provisioned, then only read. Because the driver iterates
/// *this* list rather than generating attempts as it goes, an attempt that is never reached is still
/// a known member and can be recorded
/// [`NotRun`](super::not_run_reason::NotRunReason) — which is what makes "report generation fails on
/// missing or duplicate planned identities" checkable at all.
///
/// The stored order *is* the execution order. Persisting it rather than recomputing at read time
/// means the ledger shows the order actually used even if the derivation later changes.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AttemptInventory {
    order: Vec<AttemptKey>,
}

impl AttemptInventory {
    /// Freeze the inventory in the counterbalanced, seed-permuted execution order.
    ///
    /// Two derivations with different characters, deliberately. Rung order is *counterbalanced by
    /// block parity* rather than seeded, because counterbalancing is a design property the inventory
    /// must exhibit at every seed — a coin flip would only deliver it on average, and at two blocks
    /// would fail outright a quarter of the time. Target order within a `(block, rung)` group is
    /// seeded, because there the goal is only to avoid a fixed position effect.
    pub(crate) fn frozen(seed: ScheduleSeed) -> Result<Self> {
        let mut order = Vec::with_capacity(SCREEN_ATTEMPT_COUNT);
        for block in ScreenBlockIndex::ALL {
            for rung in counterbalanced_rungs(block) {
                for target in ordered_targets(block, rung, seed) {
                    order.push(AttemptKey::new(
                        target.candidate(),
                        ExperimentAxis::UnrelatedGlobalRows,
                        rung,
                        target.role(),
                        StageRepetition::Screen(block),
                        RetryOrdinal::ORIGINAL,
                        CONTROL_REGISTRY_DISCOVERY_VERSION,
                    ));
                }
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
    /// Duplicate-slot rejection is the half that matters: a permutation bug emitting one
    /// `(block, rung)` group twice and another never would still produce sixteen attempts, so a bare
    /// length check would pass it and the screen would silently measure the wrong design.
    fn sealed(order: Vec<AttemptKey>) -> Result<Self> {
        ensure!(
            order.len() == SCREEN_ATTEMPT_COUNT,
            "a frozen screen inventory must predeclare exactly {SCREEN_ATTEMPT_COUNT} attempts, \
             got {}",
            order.len(),
        );
        for (position, attempt) in order.iter().enumerate() {
            for (other_position, other) in order.iter().enumerate().skip(position + 1) {
                ensure!(
                    !attempt.same_logical_slot(*other),
                    "the frozen screen inventory repeats a logical slot at positions {position} \
                     and {other_position}; every predeclared attempt must address a distinct slot",
                );
            }
        }
        Ok(Self { order })
    }
}

/// This block's two rungs in counterbalanced execution order.
///
/// Even blocks run low before high, odd blocks high before low, so across the pair each endpoint
/// runs first exactly once and neither rung is confounded with chronological position. Returned as
/// one array so the pair's completeness is structural: both rungs always appear, only their order
/// varies.
fn counterbalanced_rungs(block: ScreenBlockIndex) -> [ScreenRung; 2] {
    if block.runs_low_rung_first() {
        [ScreenRung::Low, ScreenRung::High]
    } else {
        [ScreenRung::High, ScreenRung::Low]
    }
}

/// The four targets of one `(block, rung)` group, permuted by a domain-separated digest.
///
/// Sorting a `Vec` is a permutation, so the result is exactly [`ScreenTarget::ALL`] reordered — no
/// target added, dropped, or duplicated. The digest subject is built from stable values — never
/// `Debug` output — so the derived order is reproducible against a recorded seed and cannot be moved
/// by a Rust rename.
fn ordered_targets(
    block: ScreenBlockIndex,
    rung: ScreenRung,
    seed: ScheduleSeed,
) -> Vec<ScreenTarget> {
    let mut keyed: Vec<(ScreenTargetOrderKey, ScreenTarget)> = ScreenTarget::ALL
        .into_iter()
        .map(|target| (target_order_key(block, rung, target, seed), target))
        .collect();
    keyed.sort_by(|(left, _), (right, _)| left.cmp(right));
    keyed.into_iter().map(|(_, target)| target).collect()
}

/// The total ordering key for one target under `seed`.
fn target_order_key(
    block: ScreenBlockIndex,
    rung: ScreenRung,
    target: ScreenTarget,
    seed: ScheduleSeed,
) -> ScreenTargetOrderKey {
    let tag = target.canonical_tag();
    let subject = format!(
        "{};seed={};block={};rung={};target={}",
        SCREEN_TARGET_ORDER_DOMAIN,
        seed.get(),
        block.get(),
        rung.canonical_tag(),
        tag,
    );
    let mut hasher = Sha256::new();
    hasher.update(subject.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    (digest, tag)
}

#[cfg(test)]
mod tests;
