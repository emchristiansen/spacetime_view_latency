//! Which retry of a logical attempt slot this attempt is.

use serde::Serialize;

/// The 0-based retry ordinal distinguishing an original attempt from its retries.
///
/// The spec requires that "retries mint new identities and never overwrite evidence" and that
/// analysis "selects exactly one current valid complete attempt per logical slot while the ledger
/// still displays originals, failures, and retries". This ordinal is the component of
/// [`AttemptKey`](super::attempt_key::AttemptKey) that makes those two requirements compatible: the
/// logical slot is the key *without* this field, and the durable identity is the key *with* it, so a
/// retry is a genuinely new record rather than an overwrite.
///
/// [`Self::ORIGINAL`] is the first attempt at a slot; [`Self::next`] mints its successor. There is no
/// arbitrary constructor, so an ordinal can only be reached by counting up from the original — a
/// retry cannot be minted out of thin air at some arbitrary number that would imply retries that
/// never happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct RetryOrdinal(u32);

impl RetryOrdinal {
    /// The original, non-retry attempt at a logical slot.
    pub(crate) const ORIGINAL: RetryOrdinal = RetryOrdinal(0);

    /// The next retry after this one. Saturating is deliberately not used: a `u32` overflow here
    /// would mean four billion retries of a single slot, which is a runaway loop rather than a
    /// number to clamp, so it panics in debug and is unreachable in practice.
    pub(crate) fn next(self) -> Self {
        RetryOrdinal(self.0 + 1)
    }

    /// The 0-based retry number.
    pub(crate) fn get(self) -> u32 {
        self.0
    }
}
