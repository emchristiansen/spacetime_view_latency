//! Which retry of a logical attempt slot this attempt is.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its whole guarantee is
//! that only two values exist. There is no callable constructor at all, so the guarantee rests
//! entirely on who may write the tuple literal — and Rust makes a private field visible to its
//! declaring module **and every descendant**. A `#[cfg(test)] mod tests` child, or any child added
//! later, could write `RetryOrdinal(7)`: a third ordinal the protocol forbids, which the "at most one
//! retry per logical slot" rule and the lowest-ordinal selection fold both assume cannot exist.
//! `sealed` has no children, so the two associated constants below really are the only values.

mod sealed {
    use serde::Serialize;

    /// The 0-based retry ordinal distinguishing an original attempt from its one permitted retry.
    ///
    /// This is the component of
    /// [`AttemptKey`](crate::view_read_set_campaign::attempt_key::AttemptKey) that reconciles two
    /// spec requirements: retries "mint new identities and never overwrite evidence", yet analysis
    /// selects one attempt per logical slot. The slot is the key *without* this field; the durable
    /// identity is the key *with* it.
    ///
    /// Exactly two values exist, because the spec caps retries at one per logical slot: a third
    /// ordinal is not merely unused, it is a state the protocol forbids. That is enforced by this
    /// childless module rather than by convention — the field is private to it and no minting API is
    /// exposed, so [`Self::ORIGINAL`] and [`Self::RETRY`] are the only values in the crate. Analysis
    /// selects the valid complete attempt with the lowest ordinal, which [`Ord`] makes a comparison
    /// on this type rather than a convention.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    #[serde(transparent)]
    pub(crate) struct RetryOrdinal(u32);

    impl RetryOrdinal {
        /// The original, non-retry attempt at a logical slot.
        pub(crate) const ORIGINAL: RetryOrdinal = RetryOrdinal(0);

        /// The single permitted retry of a logical slot, reachable only after an environment-gate
        /// invalidation or an infrastructure failure that occurred before the original's first
        /// measured sample. If it also fails, the slot is not complete — there is no successor.
        pub(crate) const RETRY: RetryOrdinal = RetryOrdinal(1);

        /// This ordinal as its 0-based number, for the canonical identity spelling.
        ///
        /// An explicit accessor rather than `Debug` or serde output: the number reaches an
        /// artifact directory name that a reader locates evidence by, so a Rust rename or a serde
        /// attribute must not be able to move it. Same contract as
        /// [`Cell::canonical_tag`](crate::plan::cell::Cell::canonical_tag).
        ///
        /// Reading it out cannot widen the type: there is still no constructor, so the only values
        /// this can report are [`Self::ORIGINAL`]'s and [`Self::RETRY`]'s.
        pub(crate) fn get(self) -> u32 {
            self.0
        }
    }
}

pub(crate) use sealed::RetryOrdinal;
