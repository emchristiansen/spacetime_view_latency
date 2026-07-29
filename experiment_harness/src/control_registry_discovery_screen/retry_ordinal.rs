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

    /// The ordinal one past this one.
    ///
    /// **Nothing in this invocation calls it.** This screen executes exactly the sixteen frozen
    /// original identities and schedules no retry: `RetryEligibility::Retryable` is a prospective
    /// fact authorizing a separately frozen future retry inventory, never an instruction for this
    /// run to loop, and inventing a bound here would recreate the dormant campaign's retry and
    /// reconciliation machinery that the spec forbids extending.
    ///
    /// It exists so the record model can *represent* a retry identity, which is what makes the
    /// supersession rules — an original may carry no link, a retry must name an earlier ordinal —
    /// checkable rather than merely asserted for a case no test could construct.
    pub(crate) fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("a retry ordinal is bounded far below u32::MAX by any frozen inventory"),
        )
    }
}
