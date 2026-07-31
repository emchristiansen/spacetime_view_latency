//! Which experiment stage an attempt belongs to, and its repetition within that stage.

use serde::Serialize;

use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;

/// The stage an attempt belongs to, carrying its block index.
///
/// Named for the spec's `StageRepetition`, which also names `Smoke` and `Confirmatory`; only the
/// stage this module executes is declared, and the others arrive with their drivers. This stage's
/// *role* is to supply Layer-B donor vectors and prove the fresh-server plumbing — that is what
/// "calibration Pilot" names, not a separate stage.
///
/// The index lives *inside* the variant so a stage cannot be paired with another stage's block
/// index — a mismatch fails to typecheck rather than needing a runtime check. Evidence must carry
/// its stage inseparably from its identity: the spec forbids a Pilot attempt from authorizing a
/// performance conclusion, and a bare block index would not say which stage produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum StageRepetition {
    /// One of the Pilot's five randomized matched blocks.
    Pilot(PilotBlockIndex),
}

impl StageRepetition {
    /// A stable canonical token naming the stage, for the identity-derived artifact directory.
    ///
    /// Frozen independently of the variant name, as every other canonical tag is. A later stage
    /// arrives as its own variant and must be spelled here, so an unnamed stage fails to compile
    /// rather than borrowing this one's directory.
    pub(crate) fn stage_tag(self) -> &'static str {
        match self {
            Self::Pilot(..) => "pilot",
        }
    }

    /// The 0-based block index within this stage.
    ///
    /// Read out of the variant rather than stored beside it, so the block cannot be paired with a
    /// stage it does not belong to. The coordinate is mandatory in the artifact directory name: two
    /// attempts of the same candidate, axis, rung, role, version, and retry differ *only* by block,
    /// so omitting it would collide the matched blocks' evidence.
    pub(crate) fn block_index(self) -> u32 {
        match self {
            Self::Pilot(block) => block.get(),
        }
    }
}
