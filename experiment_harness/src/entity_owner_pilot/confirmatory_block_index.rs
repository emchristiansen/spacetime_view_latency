//! The 0-based index of a matched block within the Confirmatory stage.

use serde::Serialize;

use crate::params::REPETITION_BLOCKS;

/// [`REPETITION_BLOCKS`] as a `usize`, for use as an array length. Defined here rather than in
/// [`crate::params`] so this candidate adds nothing to the historical campaign's preregistered
/// parameter surface. Guarded by a compile-time round-trip assertion rather than a bare `as` cast,
/// mirroring [`crate::params::BATCH_SIZE_USIZE`].
const REPETITION_BLOCKS_USIZE: usize = {
    let as_usize = REPETITION_BLOCKS as usize;
    assert!(
        as_usize as u32 == REPETITION_BLOCKS,
        "REPETITION_BLOCKS does not fit in usize on this platform"
    );
    as_usize
};

/// The 0-based position of a matched block in the Confirmatory stage's fixed thirty-block sample.
///
/// Same structural guarantee as [`PilotBlockIndex`](super::pilot_block_index::PilotBlockIndex):
/// range validity is a property of the type because the only values in existence are those in
/// [`Self::ALL`]. It exists now so [`StageRepetition`](super::stage_repetition::StageRepetition) can
/// be the spec's complete closed stage vocabulary rather than a Pilot-only subset that would have to
/// be widened — and widening a stage enum after evidence exists under it is exactly the kind of
/// retroactive reinterpretation the spec's evidence-retention rules forbid.
///
/// The spec fixes this sample at [`REPETITION_BLOCKS`] and forbids stopping early or extending, so
/// the count is sourced from that one constant rather than restated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct ConfirmatoryBlockIndex(u32);

impl ConfirmatoryBlockIndex {
    /// The fixed, module-owned sequence of every valid Confirmatory block index,
    /// `0..REPETITION_BLOCKS` in ascending order.
    pub(crate) const ALL: [ConfirmatoryBlockIndex; REPETITION_BLOCKS_USIZE] = {
        let mut all = [ConfirmatoryBlockIndex(0); REPETITION_BLOCKS_USIZE];
        let mut i = 0usize;
        while i < REPETITION_BLOCKS_USIZE {
            all[i] = ConfirmatoryBlockIndex(i as u32);
            i += 1;
        }
        all
    };

    /// The 0-based block number.
    pub(crate) fn get(self) -> u32 {
        self.0
    }
}
