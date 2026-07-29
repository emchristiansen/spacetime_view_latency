//! Which attempt at a logical slot a record belongs to.

use serde::Serialize;

/// The retry ordinal distinguishing repeated attempts at one logical slot.
///
/// Part of the full attempt identity, so a retry is written under its own key and never overwrites
/// its predecessor; the *logical slot* is the same identity with this component disregarded. All
/// sixteen planned attempts are frozen at [`Self::ORIGINAL`] — a retry ordinal above zero exists
/// only if an attempt is later re-run under the spec's narrow retry eligibility, and is never
/// predeclared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct RetryOrdinal(u32);

impl RetryOrdinal {
    /// The original, non-retry attempt at a logical slot.
    pub(crate) const ORIGINAL: RetryOrdinal = RetryOrdinal(0);
}
