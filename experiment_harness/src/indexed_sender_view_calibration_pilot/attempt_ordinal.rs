//! Which attempt at a logical slot a record belongs to.

use serde::Serialize;

/// The attempt ordinal component of an identity — and, in this module, the only one there is.
///
/// The spec's minimal type design requires attempt identity to bind a retry ordinal, so the
/// component is present rather than dropped: a reader holding only the ledger can see which attempt
/// at a slot a line describes without knowing this module's scheduling rules.
///
/// **One variant, because this invocation is authorized to run exactly two originals and schedule
/// nothing.** An earlier draft carried a `RetryOrdinal(u32)` with a `next()` and a `Supersession`
/// link, copying the discovery screen's model so a future retry inventory would be representable.
/// That was the wrong trade here: representable is exactly what an unauthorized retry must not be,
/// and a latent capability that no frozen inventory may use is a way for one to appear later without
/// an amendment. A separately frozen retry inventory is a new vocabulary and a new SSOT
/// authorization, not a dormant constructor in this one.
///
/// With a single variant there is nothing to supersede either, so no supersession link exists in
/// this module: an original that could only ever be an original does not need a field saying so
/// twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum AttemptOrdinal {
    /// The first and only attempt at this logical slot.
    Original,
}
