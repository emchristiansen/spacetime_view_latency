//! The retained reason a durable sink became terminal.

use crate::observation::record_id::RecordId;
use crate::observation::tail_state::TailState;

/// Why a sink became terminal after a persist failure: the record whose write failed, that record's
/// [`TailState`] (definitely-not-written vs durability-ambiguous), and the diagnostic. It is retained
/// so a later write refusal or finalization reports the original cause rather than a fresh,
/// contextless error.
///
/// This records why *this process* stopped writing; it is not a claim about physical crash state.
#[derive(Clone, Debug)]
pub(crate) struct PoisonedTail {
    record: RecordId,
    tail_state: TailState,
    diagnostic: String,
}

impl PoisonedTail {
    pub(crate) fn new(record: RecordId, tail_state: TailState, diagnostic: String) -> Self {
        Self {
            record,
            tail_state,
            diagnostic,
        }
    }

    /// The record whose persist failure poisoned the sink.
    pub(crate) fn record(&self) -> RecordId {
        self.record
    }

    /// The durability classification of the poisoning record's tail.
    pub(crate) fn tail_state(&self) -> TailState {
        self.tail_state
    }

    /// The human-readable diagnostic for the poisoning failure.
    pub(crate) fn diagnostic(&self) -> &str {
        &self.diagnostic
    }
}
