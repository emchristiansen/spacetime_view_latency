//! Whether a failed slot may be attempted again.

use serde::Serialize;

/// Whether the spec's retry rule admits another attempt at this logical slot.
///
/// Recorded rather than recomputed at analysis time, so the ledger states the eligibility that was
/// actually in force. Derived by [`AttemptFailure`](super::attempt_failure::AttemptFailure) from the
/// failure's phase and its sampling progress together — never from either alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum RetryEligibility {
    /// A prospective infrastructure failure before the first measured sample. A retry replaces an
    /// attempt that measured nothing.
    Retryable,
    /// Everything else. The attempt is a terminal result at its slot.
    NotRetryable,
}
