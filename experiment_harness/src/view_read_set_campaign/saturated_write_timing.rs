//! One saturated write's issue and confirmation offsets from the batch's common origin.

use anyhow::{ensure, Result};
use serde::Serialize;

/// When one write of a saturated batch was issued and when its confirmation arrived, both measured
/// as offsets from the *same* origin.
///
/// A latency alone would not do. The spec requires recording issue and confirmation offsets from a
/// common origin "so the FIFO/stable-issue-spacing interpretation is falsifiable": the saturated
/// channel's estimand is only the per-write service time if writes were in fact issued back-to-back
/// and confirmed in order, and a vector of differences cannot distinguish that from a run whose
/// issue spacing drifted. Retaining both offsets keeps the interpretation checkable against the
/// evidence instead of assumed by the analysis.
///
/// The two offsets share one origin per batch, so they are directly comparable across writes —
/// which is what makes issue spacing and confirmation ordering recoverable at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct SaturatedWriteTiming {
    issue_offset_nanos: u128,
    confirmation_offset_nanos: u128,
}

impl SaturatedWriteTiming {
    /// Record one write's offsets, failing loud unless its confirmation is at or after its issue.
    ///
    /// An inverted pair is not a slow write but a broken clock or a mismatched origin, and it would
    /// silently produce a negative latency that the slope reduction would then average in. Rejecting
    /// it here means [`Self::latency_nanos`] is total.
    ///
    /// This is a well-formedness constraint, not evidence of measurement: any caller may supply any
    /// ordered pair. What the type guarantees is that no *ill-formed* pair reaches the reduction.
    pub(crate) fn observed(issue_offset_nanos: u128, confirmation_offset_nanos: u128) -> Result<Self> {
        ensure!(
            confirmation_offset_nanos >= issue_offset_nanos,
            "a saturated write's confirmation offset ({confirmation_offset_nanos} ns) precedes its \
             issue offset ({issue_offset_nanos} ns); both must be measured from the same origin"
        );
        Ok(Self {
            issue_offset_nanos,
            confirmation_offset_nanos,
        })
    }

    /// When this write was issued, relative to the batch's common origin. Consecutive issue offsets
    /// are what make the back-to-back issue claim falsifiable.
    pub(crate) fn issue_offset_nanos(self) -> u128 {
        self.issue_offset_nanos
    }

    /// When this write's confirmation arrived, relative to the same origin. The ordering of these
    /// across a batch is what makes the FIFO claim falsifiable.
    pub(crate) fn confirmation_offset_nanos(self) -> u128 {
        self.confirmation_offset_nanos
    }

    /// This write's confirmed round-trip latency. Total: [`Self::observed`] refuses an inverted
    /// pair, so the subtraction never underflows.
    pub(crate) fn latency_nanos(self) -> u128 {
        self.confirmation_offset_nanos - self.issue_offset_nanos
    }
}
