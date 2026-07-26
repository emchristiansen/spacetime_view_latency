//! The frozen, ordered inventory of every predeclared attempt of one candidate/axis campaign.

use anyhow::{ensure, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::campaign_params::{
    CAMPAIGN_SEED, PILOT_BLOCKS_USIZE, RUNG_ORDER_DOMAIN, UNRELATED_GLOBAL_ROWS_LADDER_LEN,
};
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

/// Total ordering key for one rung of the base permutation: its SHA-256 digest, then its ladder
/// position as a deterministic tiebreak, so the induced order is total even on a digest tie.
/// Mirrors the historical `BlockOrderKey` (`plan::schedule::schedule`) and the completed Pilot's
/// `PilotBlockOrderKey`.
type RungOrderKey = ([u8; 32], usize);

/// How many attempts this campaign predeclares: every block, every rung, both roles.
///
/// Five blocks × six rungs × two roles = sixty, and it is written as that product rather than as a
/// literal so a change to either frozen input moves it. This is `5 × 2 × r` in the spec's terms —
/// the fresh-server count for a five-block calibration Pilot on a six-point ladder.
pub(crate) const CAMPAIGN_ATTEMPT_COUNT: usize =
    PILOT_BLOCKS_USIZE * UNRELATED_GLOBAL_ROWS_LADDER_LEN * 2;

/// The complete, frozen, execution-ordered list of every attempt this campaign will make.
///
/// Built once before the first server is provisioned, then only read. Because the driver iterates
/// *this* list rather than generating attempts as it goes, an attempt that is never reached is still
/// a known member and can be recorded
/// [`NotRun`](super::attempt_outcome::AttemptOutcome::NotRun) — which is what makes "report
/// generation fails on missing or duplicate planned identities" checkable.
///
/// The stored order *is* the execution order. Persisting it rather than recomputing at read time
/// means the ledger shows the order actually used even if the derivation later changes. It is
/// inventory metadata: analysis reconstructs ladders by scale-point identity, never by ledger order,
/// and the adjacency of a rung's two roles is not an inferential pairing.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AttemptInventory {
    order: Vec<AttemptKey>,
}

impl AttemptInventory {
    /// Freeze the inventory in the preregistered execution order.
    ///
    /// Takes no seed: unlike the completed Pilot, whose block order came from a CLI argument, this
    /// campaign's order is derived from the frozen [`CAMPAIGN_SEED`], so there is no run-time input
    /// that could produce a different preregistration.
    ///
    /// The order is the spec's, in three layers:
    ///
    /// 1. **Blocks ascend.** No permutation — the randomization that matters is within the block.
    /// 2. **Rungs follow that block's cyclic order.** One base permutation is derived from the
    ///    frozen seed under [`RUNG_ORDER_DOMAIN`], then block `b` rotates it by `b mod r`. Every
    ///    rung therefore occupies each execution position within one of at most `⌈blocks/r⌉` counts
    ///    of each other — exact balance when the block count divides by the ladder length, and the
    ///    closest possible balance otherwise, without moving the frozen block count.
    /// 3. **The two roles of a rung are adjacent**, with the first alternating by block parity —
    ///    Arm first on even blocks, Control first on odd. Each role independently walks the block's
    ///    prescribed rung sequence, matched roles never drift apart across the campaign, and
    ///    first-role counts are exactly balanced for an even block count and differ by one for the
    ///    five-block Pilot.
    pub(crate) fn frozen() -> Result<Self> {
        let axis = ExperimentAxis::UnrelatedGlobalRows;
        let scales = ScalePoint::ladder(axis);
        let base = base_rung_order(scales.len());

        let mut order = Vec::with_capacity(CAMPAIGN_ATTEMPT_COUNT);
        for block in PilotBlockIndex::ALL {
            for position in cyclic_rung_order(&base, block) {
                for role in ordered_roles(block) {
                    order.push(AttemptKey::new(
                        CandidateId::EntityOwnerSenderView,
                        scales[position],
                        role,
                        StageRepetition::Pilot(block),
                        RetryOrdinal::ORIGINAL,
                        ENTITY_OWNER_SENDER_VIEW_VERSION,
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
    /// Duplicate-slot rejection is the half that matters: a permutation bug emitting one rung twice
    /// and another never would still produce sixty attempts, so a bare length check would pass it
    /// and the campaign would silently measure the wrong design.
    fn sealed(order: Vec<AttemptKey>) -> Result<Self> {
        ensure!(
            order.len() == CAMPAIGN_ATTEMPT_COUNT,
            "a frozen campaign inventory must predeclare exactly {CAMPAIGN_ATTEMPT_COUNT} \
             attempts, got {}",
            order.len(),
        );
        for (position, attempt) in order.iter().enumerate() {
            for (other_position, other) in order.iter().enumerate().skip(position + 1) {
                ensure!(
                    !attempt.same_logical_slot(*other),
                    "the frozen campaign inventory repeats a logical slot at positions {position} \
                     and {other_position}; every predeclared attempt must address a distinct slot",
                );
            }
        }
        Ok(Self { order })
    }
}

/// The base rung permutation, derived once from the frozen campaign seed.
///
/// Copies [`Schedule::seeded_permutation`](crate::plan::schedule::Schedule): each ladder position is
/// assigned a [`RungOrderKey`] and the vector is sorted by it. Sorting a `Vec` is a permutation, so
/// the result is exactly `0..len` reordered — no position added, dropped, or duplicated.
fn base_rung_order(len: usize) -> Vec<usize> {
    let mut keyed: Vec<(RungOrderKey, usize)> = (0..len)
        .map(|position| (rung_order_key(position), position))
        .collect();
    keyed.sort_by(|(left, _), (right, _)| left.cmp(right));
    keyed.into_iter().map(|(_, position)| position).collect()
}

/// The total ordering key for one ladder position under the frozen seed.
///
/// The subject's byte encoding is itself preregistered, and copies
/// [`Schedule::block_order_key`](crate::plan::schedule::Schedule) exactly: semicolon-separated
/// `key=value` coordinates following the domain, digested as UTF-8. It is built from stable numeric
/// values — never `Debug` output — so the derived order is reproducible against the recorded seed
/// and cannot be moved by a Rust rename.
fn rung_order_key(position: usize) -> RungOrderKey {
    let subject = format!(
        "{};seed={};rung={}",
        RUNG_ORDER_DOMAIN, CAMPAIGN_SEED, position
    );
    let mut hasher = Sha256::new();
    hasher.update(subject.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    (digest, position)
}

/// One block's rung order: the base permutation rotated by `block_index mod r`.
///
/// A rotation rather than a fresh permutation per block is what makes the design *near-balanced* by
/// construction: across blocks, each rung visits execution positions in a cycle, so every position
/// count differs by at most one. A per-block reshuffle would only be balanced on average.
fn cyclic_rung_order(base: &[usize], block: PilotBlockIndex) -> Vec<usize> {
    let len = base.len();
    let block_index = usize::try_from(block.get()).expect("a frozen block index fits usize");
    let shift = block_index % len;
    (0..len).map(|offset| base[(offset + shift) % len]).collect()
}

/// One rung's Arm and matched Control in execution order, returned together as one array so the
/// pair's adjacency is structural rather than caller-maintained.
///
/// The pair is always exactly `{Arm, Control}`; only the order varies, alternating by block parity.
/// Parity rather than a seeded bit because the requirement is *balance* across blocks, and a derived
/// bit would only balance in expectation.
fn ordered_roles(block: PilotBlockIndex) -> [RunRole; 2] {
    if block.get() % 2 == 0 {
        [RunRole::Arm, RunRole::Control]
    } else {
        [RunRole::Control, RunRole::Arm]
    }
}
