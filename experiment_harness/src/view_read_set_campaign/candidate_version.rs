//! The implementation version of the candidate an attempt exercised.
//!
//! The type and the one constant that mints it live together in the private, *childless* inline
//! module [`sealed`]. There is no callable constructor, so the type's guarantee rests entirely on who
//! may write the tuple literal — and Rust makes a private field visible to its declaring module **and
//! every descendant**. A `#[cfg(test)] mod tests` child, or any child added later, could write
//! `CandidateVersion(2)` and silently split evidence that belongs to one measured path into two
//! version lines, which is exactly what binding the version into every identity exists to prevent.
//! The constant is sealed *with* the type because it is the sole minting expression.

mod sealed {
    use serde::Serialize;

    /// The monotonically increasing implementation version of a candidate's measured code path.
    ///
    /// Binding it into every attempt identity is what stops evidence produced by materially
    /// different code from being pooled as one sample. It is deliberately not derived from the git
    /// commit: a formatting-only commit must not invalidate evidence, and a semantic change made
    /// without a commit must not appear valid. Commit provenance is recorded separately by the run's
    /// build provenance.
    ///
    /// The field is private to this childless module and no minting API is exposed, so
    /// [`ENTITY_OWNER_SENDER_VIEW_VERSION`] is the only value of this type in the crate — a version
    /// nothing declared cannot be attached to an attempt.
    ///
    /// Numbered independently of the completed seed-7 Pilot's version of the same name. That
    /// campaign measured a cumulative walk on one server; this one measures a fixed-cardinality
    /// scale point on a fresh server, so the two are different measured paths and their version
    /// lines do not continue each other.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    #[serde(transparent)]
    pub(crate) struct CandidateVersion(u32);

    impl CandidateVersion {
        /// This version as its number, for the canonical identity spelling.
        ///
        /// An explicit accessor rather than `Debug` or serde output, for the same reason
        /// [`RetryOrdinal::get`](crate::view_read_set_campaign::retry_ordinal::RetryOrdinal::get)
        /// is one: the number reaches an artifact directory name a reader locates evidence by.
        /// Reading it out adds no constructor, so the declared version constants remain the only
        /// values in existence.
        pub(crate) fn get(self) -> u32 {
            self.0
        }
    }

    /// The current fresh-server implementation version of the `EntityOwnerSenderView` candidate.
    ///
    /// Bump when the measured path's semantics change — the view body, the seeded composition, the
    /// subscription shape, the mutation under measurement, or what a channel's measured interval
    /// covers — and never for a formatting or comment-only change.
    pub(crate) const ENTITY_OWNER_SENDER_VIEW_VERSION: CandidateVersion = CandidateVersion(1);
}

pub(crate) use sealed::{CandidateVersion, ENTITY_OWNER_SENDER_VIEW_VERSION};
