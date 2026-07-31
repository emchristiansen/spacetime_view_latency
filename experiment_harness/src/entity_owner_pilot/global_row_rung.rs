//! One rung of the frozen `N_global` ladder.

use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::{GLOBAL_ROW_LADDER, GLOBAL_ROW_LADDER_LEN};

/// The 0-based position of one rung in the frozen `N_global` ladder.
///
/// Proves range validity only, by the same construction as
/// [`PilotBlockIndex`](super::pilot_block_index::PilotBlockIndex): a private field, no minting API,
/// and [`Self::ALL`] as the sole source of values. Consuming `ALL` in order is how an attempt walks
/// the ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct GlobalRowRung(usize);

impl GlobalRowRung {
    /// Every valid rung in ascending ladder order — the only values in existence.
    pub(crate) const ALL: [GlobalRowRung; GLOBAL_ROW_LADDER_LEN] = {
        let mut all = [GlobalRowRung(0); GLOBAL_ROW_LADDER_LEN];
        let mut i = 0usize;
        while i < GLOBAL_ROW_LADDER_LEN {
            all[i] = GlobalRowRung(i);
            i += 1;
        }
        all
    };

    /// The 0-based rung number.
    pub(crate) fn get(self) -> usize {
        self.0
    }

    /// This rung's preregistered global row count. Total: the index is in range by construction.
    pub(crate) fn global_rows(self) -> u64 {
        GLOBAL_ROW_LADDER[self.0]
    }

    /// How many global rows this rung adds on top of the previous one.
    ///
    /// The ladder is cumulative and walked progressively on a single server, so a rung writes only
    /// the difference. Deriving that here keeps the cumulative invariant in one place: a driver
    /// cannot accidentally re-seed the full count and double the backing table.
    pub(crate) fn global_rows_increment(self) -> u64 {
        match self.0.checked_sub(1) {
            None => GLOBAL_ROW_LADDER[0],
            Some(previous) => GLOBAL_ROW_LADDER[self.0] - GLOBAL_ROW_LADDER[previous],
        }
    }
}
