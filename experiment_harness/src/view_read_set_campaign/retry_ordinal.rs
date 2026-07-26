//! Which retry of a logical attempt slot this attempt is.

use serde::Serialize;

/// The 0-based retry ordinal distinguishing an original attempt from its one permitted retry.
///
/// This is the component of [`AttemptKey`](super::attempt_key::AttemptKey) that reconciles two spec
/// requirements: retries "mint new identities and never overwrite evidence", yet analysis selects
/// one attempt per logical slot. The slot is the key *without* this field; the durable identity is
/// the key *with* it.
///
/// Exactly two values exist, because the spec caps retries at one per logical slot: a third ordinal
/// is not merely unused, it is a state the protocol forbids. Analysis selects the valid complete
/// attempt with the lowest ordinal, which [`Ord`] makes a comparison on this type rather than a
/// convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct RetryOrdinal(u32);

impl RetryOrdinal {
    /// The original, non-retry attempt at a logical slot.
    pub(crate) const ORIGINAL: RetryOrdinal = RetryOrdinal(0);

    /// The single permitted retry of a logical slot, reachable only after an environment-gate
    /// invalidation or an infrastructure failure that occurred before the original's first measured
    /// sample. If it also fails, the slot is not complete — there is no successor.
    pub(crate) const RETRY: RetryOrdinal = RetryOrdinal(1);
}
