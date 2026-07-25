//! Which experiment stage an attempt belongs to, and its repetition within that stage.

use serde::Serialize;

use crate::entity_owner_pilot::confirmatory_block_index::ConfirmatoryBlockIndex;
use crate::entity_owner_pilot::pilot_block_index::PilotBlockIndex;

/// The stage an attempt belongs to, carrying its block index where the stage is a repeated one.
///
/// Transcribed verbatim from the spec's "Minimal type design" `StageRepetition`. Making the block
/// index part of the stage variant — rather than a separate field beside a stage tag — is what makes
/// the invalid combinations unrepresentable: a Smoke attempt cannot carry a block index (Smoke is
/// "one plumbing and semantic sanity execution"), and a Pilot attempt cannot carry a Confirmatory
/// block index or vice versa. A stage/index mismatch is therefore not a runtime check that could be
/// forgotten at a call site; it does not typecheck.
///
/// This distinction is load-bearing for analysis: the spec forbids a Pilot attempt from authorizing
/// a performance conclusion, so evidence must carry its stage inseparably from its identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum StageRepetition {
    /// The single plumbing and semantic sanity execution. Unrepeated, so it carries no index.
    Smoke,
    /// One of the Pilot's five randomized matched blocks. Records raw evidence and data adequacy
    /// only — never a performance conclusion.
    Pilot(PilotBlockIndex),
    /// One of the Confirmatory stage's thirty randomized matched blocks — the scientific decision.
    Confirmatory(ConfirmatoryBlockIndex),
}
