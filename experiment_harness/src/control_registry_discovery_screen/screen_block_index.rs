//! The independent blocks this screen runs.

use serde::Serialize;

use crate::control_registry_discovery_screen::screen_params::SCREEN_BLOCKS_USIZE;

/// A validated block index. [`Self::ALL`] is the only source of values, so a block outside the
/// frozen count is unrepresentable rather than merely unwritten.
///
/// Block parity is load-bearing here, not decorative: it selects the rung order, which is how the
/// screen counterbalances chronology against rung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct ScreenBlockIndex(u32);

impl ScreenBlockIndex {
    /// Every valid block index in ascending order — the only values in existence.
    pub(crate) const ALL: [ScreenBlockIndex; SCREEN_BLOCKS_USIZE] = {
        let mut all = [ScreenBlockIndex(0); SCREEN_BLOCKS_USIZE];
        let mut i = 0usize;
        while i < SCREEN_BLOCKS_USIZE {
            all[i] = ScreenBlockIndex(i as u32);
            i += 1;
        }
        all
    };

    /// The 0-based block number.
    pub(crate) fn get(self) -> u32 {
        self.0
    }

    /// Whether this block runs the low rung first.
    ///
    /// Alternation by parity rather than a seeded coin: counterbalancing is a *design* property the
    /// inventory must exhibit, so it must hold at every seed, not merely on average across seeds.
    pub(crate) fn runs_low_rung_first(self) -> bool {
        self.0 % 2 == 0
    }
}
