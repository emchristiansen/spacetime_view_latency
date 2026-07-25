//! The 0-based index of a matched block within the Pilot stage.

use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::PILOT_BLOCKS_USIZE;

/// The 0-based position of a matched block in the Pilot's fixed five-block sample.
///
/// Proves range validity only. There is no callable constructor — the private field plus the absence
/// of any minting API means the sole source of values is [`Self::ALL`], so range membership is a
/// property of the type rather than of caller discipline. Mirrors
/// [`DoseIndex`](crate::dataset::dose_index::DoseIndex).
///
/// Says nothing about *when* its block runs: the execution order is the seeded permutation held by
/// [`AttemptInventory`](super::attempt_inventory::AttemptInventory), which is why the ledger records
/// both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct PilotBlockIndex(u32);

impl PilotBlockIndex {
    /// Every valid block index in ascending order — the only values in existence.
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
