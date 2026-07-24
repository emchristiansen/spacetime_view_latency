//! How a failed record's bytes stand relative to durable storage.

/// The durability classification of the record whose persist failed. A two-state enum — never a
/// bool — so callers pattern-match exhaustively and cannot invert or forget the meaning, and it maps
/// directly onto the campaign's `SinkWriteState` without a boolean bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TailState {
    /// A pre-write (serialization) failure: no bytes were written, so the record is definitely
    /// absent.
    DefinitelyNotWritten,
    /// A write/flush/sync failure after serialization: a full line may already be durable, so the
    /// record's durability is unknown.
    DurabilityAmbiguous,
}
