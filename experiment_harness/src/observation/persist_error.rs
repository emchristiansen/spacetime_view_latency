//! Why a durable sink write did not complete, carrying enough to classify the sink's tail state.

use crate::observation::poisoned_tail::PoisonedTail;
use crate::observation::record_id::RecordId;
use crate::observation::tail_state::TailState;

/// Why a durable write did not complete.
///
/// A fresh attempt fails in one of two ways: serialization fails *before* any write, so the record is
/// definitely absent ([`Self::BeforeWrite`]); or a write/flush/sync step fails *after* serialization,
/// so a full line may already be durable and the record's durability is ambiguous
/// ([`Self::DurabilityAmbiguous`]). A third case is a *refusal*: once the sink is poisoned by a prior
/// failure it refuses further writes ([`Self::SinkPoisoned`]) — that successor wrote nothing and has
/// no attempted [`RecordId`] of its own, so it must never be misread as a fresh attempt; the first
/// failure's typed state is retained in `original`. This is the distinction a run frontier needs to
/// set its `SinkWriteState` (in the not-yet-wired campaign module).
#[derive(Debug)]
pub(crate) enum PersistError {
    /// The record could not be serialized; no bytes were written and it is definitely absent.
    BeforeWrite { record: RecordId, diagnostic: String },
    /// A write/flush/sync step failed after serialization; the record's durability is unknown.
    DurabilityAmbiguous { record: RecordId, diagnostic: String },
    /// The write was refused because a prior failure had already poisoned the sink. The refused
    /// successor wrote nothing and has no attempted record; the original poison reason is retained.
    SinkPoisoned { original: PoisonedTail },
}

impl PersistError {
    /// The record this call attempted to write, if it attempted one. A refused write
    /// ([`Self::SinkPoisoned`]) wrote nothing and has no attempted record — higher layers must not
    /// treat a refusal as a fresh attempt.
    pub(crate) fn attempted_record(&self) -> Option<RecordId> {
        match self {
            PersistError::BeforeWrite { record, .. }
            | PersistError::DurabilityAmbiguous { record, .. } => Some(*record),
            PersistError::SinkPoisoned { .. } => None,
        }
    }

    /// The retained original poison reason, if this is a refusal of an already-poisoned sink.
    pub(crate) fn poisoned_original(&self) -> Option<&PoisonedTail> {
        match self {
            PersistError::SinkPoisoned { original } => Some(original),
            _ => None,
        }
    }

    /// The human-readable diagnostic for the failure.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            PersistError::BeforeWrite { diagnostic, .. }
            | PersistError::DurabilityAmbiguous { diagnostic, .. } => diagnostic,
            PersistError::SinkPoisoned { original } => original.diagnostic(),
        }
    }

    /// The sink tail's durability classification: for a fresh failure, this record's state; for a
    /// refusal, the retained original failure's state (the refusal itself wrote nothing). Maps
    /// directly onto the campaign's `SinkWriteState` without a boolean bridge.
    pub(crate) fn tail_state(&self) -> TailState {
        match self {
            PersistError::BeforeWrite { .. } => TailState::DefinitelyNotWritten,
            PersistError::DurabilityAmbiguous { .. } => TailState::DurabilityAmbiguous,
            PersistError::SinkPoisoned { original } => original.tail_state(),
        }
    }
}
