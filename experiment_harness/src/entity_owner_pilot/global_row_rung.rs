//! One rung of the frozen `N_global` ladder.

use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::{GLOBAL_ROW_LADDER, GLOBAL_ROW_LADDER_LEN};

/// The 0-based position of one rung in the frozen `N_global` ladder.
///
/// Proves **range validity only**, by the same construction as
/// [`DoseIndex`](crate::dataset::dose_index::DoseIndex) and
/// [`PilotBlockIndex`](super::pilot_block_index::PilotBlockIndex): a private field, no minting API,
/// and a module-owned [`Self::ALL`] as the sole source of values. A rung outside the frozen ladder
/// is therefore unrepresentable rather than rejected at runtime.
///
/// [`Self::global_rows`] is the single canonical mapping from rung to preregistered row count, so
/// the runtime driver and any later analysis read the ladder from one place and cannot drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct GlobalRowRung(usize);

impl GlobalRowRung {
    /// The fixed, module-owned sequence of every valid rung, in ascending ladder order. Consuming
    /// this array in array order is how an attempt walks the ladder progressively.
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

    /// This rung's preregistered unrelated/global row count, read from [`GLOBAL_ROW_LADDER`]. The
    /// index is in range by construction, so this is a total function with no fallible path.
    pub(crate) fn global_rows(self) -> u64 {
        GLOBAL_ROW_LADDER[self.0]
    }

    /// The number of additional global rows this rung seeds on top of the previous rung — the
    /// *increment* an attempt actually writes when it steps the ladder progressively on one server.
    ///
    /// The ladder is cumulative: rung 0 seeds its full count, and each later rung seeds only the
    /// difference. Deriving the increment here rather than at the call site keeps the cumulative
    /// invariant in one place, so a driver cannot accidentally re-seed the whole rung and double the
    /// backing table.
    pub(crate) fn global_rows_increment(self) -> u64 {
        match self.0.checked_sub(1) {
            None => GLOBAL_ROW_LADDER[0],
            Some(previous) => GLOBAL_ROW_LADDER[self.0] - GLOBAL_ROW_LADDER[previous],
        }
    }
}
