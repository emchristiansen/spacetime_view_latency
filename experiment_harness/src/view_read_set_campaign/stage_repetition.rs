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
