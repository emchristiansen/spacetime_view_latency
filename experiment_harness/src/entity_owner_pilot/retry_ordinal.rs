//! Which retry of a logical attempt slot this attempt is.

use serde::Serialize;

/// The 0-based retry ordinal distinguishing an original attempt from its retries.
///
/// This is the component of [`AttemptKey`](super::attempt_key::AttemptKey) that reconciles two spec
/// requirements: retries "mint new identities and never overwrite evidence", yet analysis selects
/// one attempt "per logical slot". The slot is the key *without* this field; the durable identity is
/// the key *with* it.
///
/// [`Self::ORIGINAL`] is currently the only reachable value, so no code path can mint a nonzero
/// ordinal implying retries that never happened. A successor constructor arrives with the retry
/// driver that needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct RetryOrdinal(u32);

impl RetryOrdinal {
    /// The original, non-retry attempt at a logical slot.
    pub(crate) const ORIGINAL: RetryOrdinal = RetryOrdinal(0);
}
