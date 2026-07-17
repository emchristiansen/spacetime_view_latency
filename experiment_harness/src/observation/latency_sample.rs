//! One confirmed write's issue-to-confirm round-trip latency.

use std::time::Duration;

use serde::Serialize;

/// A single measured confirmed round-trip latency, stored in nanoseconds as `u128` — the exact
/// width of [`Duration::as_nanos`], so a sample never narrows or overflows when captured. Anton's
/// pipeline records one of these per write in a dose batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(crate) struct LatencySample {
    nanos: u128,
}

impl LatencySample {
    /// Capture a measured elapsed duration losslessly (`Duration::as_nanos` is already `u128`, so
    /// there is no narrowing cast).
    pub(crate) fn from_elapsed(elapsed: Duration) -> Self {
        Self {
            nanos: elapsed.as_nanos(),
        }
    }

    /// The round-trip latency in nanoseconds.
    pub(crate) fn nanos(self) -> u128 {
        self.nanos
    }
}
