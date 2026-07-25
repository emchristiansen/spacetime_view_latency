//! The 0-based index of a matched block within the Pilot stage.

use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::PILOT_BLOCKS_USIZE;

/// The 0-based position of a matched block in the Pilot's fixed five-block sample.
///
/// Proves **range validity only**: every `PilotBlockIndex` that exists is a member of
/// `0..PILOT_BLOCKS`. It has no callable constructor — the private field plus the absence of any
/// minting API means the sole source of values is the module-owned fixed array [`Self::ALL`], so no
/// caller can fabricate one and range membership is a property of the type, not of caller
/// discipline. This mirrors [`DoseIndex`](crate::dataset::dose_index::DoseIndex) exactly.
///
/// *Execution* order is a separate concern: [`Self::ALL`] is the canonical block identity in
/// ascending order, while the order blocks are actually run in is the seeded permutation derived by
/// [`AttemptInventory`](super::attempt_inventory::AttemptInventory). The index says nothing about
/// when its block runs, which is exactly why the durable ledger records both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct PilotBlockIndex(u32);

impl PilotBlockIndex {
    /// The fixed, module-owned sequence of every valid Pilot block index, `0..PILOT_BLOCKS` in
    /// ascending order. Built in a `const` block from the private field, so these are the only
    /// `PilotBlockIndex` values in existence and there is no runtime minting path.
    pub(crate) const ALL: [PilotBlockIndex; PILOT_BLOCKS_USIZE] = {
        let mut all = [PilotBlockIndex(0); PILOT_BLOCKS_USIZE];
        let mut i = 0usize;
        while i < PILOT_BLOCKS_USIZE {
            all[i] = PilotBlockIndex(i as u32);
            i += 1;
        }
        all
    };

    /// The 0-based block number.
    pub(crate) fn get(self) -> u32 {
        self.0
    }
}
