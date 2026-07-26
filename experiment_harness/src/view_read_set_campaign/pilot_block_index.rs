//! The 0-based index of a matched block within the Pilot stage.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its range guarantee is
//! a claim about *who may write the tuple literal*, there being no callable constructor. Rust makes a
//! private field visible to its declaring module **and every descendant**, so a `#[cfg(test)] mod
//! tests` child — or any child added later — could write `PilotBlockIndex(9)` and produce a block
//! outside the frozen five-block sample. That index also selects a `(block, role)`'s cyclic rung
//! shift, so an out-of-range one would silently derive an execution order the preregistration never
//! named. `sealed` has no children, so [`PilotBlockIndex::ALL`] really is the only source of values.

mod sealed {
    use serde::Serialize;

    use crate::view_read_set_campaign::campaign_params::PILOT_BLOCKS_USIZE;

    /// The 0-based position of a matched block in the Pilot stage's fixed five-block sample.
    ///
    /// Proves range validity only. There is no callable constructor — the private field, confined to
    /// this childless module, plus the absence of any minting API means the sole source of values is
    /// [`Self::ALL`], so range membership is a property of the type rather than of caller
    /// discipline, and remains one under anything added to this file afterwards. Mirrors
    /// [`PilotBlockIndex`](crate::entity_owner_pilot::pilot_block_index::PilotBlockIndex), the
    /// same-named index of the completed cumulative Pilot; the two are separate types because they
    /// index separate campaigns' evidence.
    ///
    /// The block index is also what selects a `(block, role)`'s cyclic rung shift, so it is a
    /// preregistered coordinate rather than a mere loop counter.
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
}

pub(crate) use sealed::PilotBlockIndex;
