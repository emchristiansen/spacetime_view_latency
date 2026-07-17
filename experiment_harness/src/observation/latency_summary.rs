//! The per-dose median and IQR, derived from the lossless raw latencies.

use serde::Serialize;

use crate::observation::raw_latencies::RawLatencies;

/// The per-dose summary statistics reported alongside the raw vector: the median and interquartile
/// range of the confirmed round-trip latencies, in nanoseconds.
///
/// The sole constructor is [`Self::from_raw`], so construction is centralized and the summary is
/// derived from its dose's [`RawLatencies`] at record assembly rather than supplied independently.
/// This does not by itself prove the summary is mathematically correct or that a bug in a future
/// `from_raw` could not produce a value inconsistent with the raw vector — that is established by
/// tests added when the derivation is implemented. The exact quartile/IQR convention (e.g. the
/// interpolation rule) is deliberately **not** chosen here: it is selected in the measurement
/// milestone. Picking one silently now would preempt that decision, so the derivation is `todo!()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct LatencySummary {
    median_nanos: u128,
    iqr_nanos: u128,
}

impl LatencySummary {
    /// Derive the median and IQR from the raw latencies — the only way to build a summary.
    pub(crate) fn from_raw(_raw: &RawLatencies) -> Self {
        todo!("the exact quartile/IQR convention is selected in the measurement milestone")
    }

    /// The median round-trip latency in nanoseconds.
    pub(crate) fn median_nanos(self) -> u128 {
        self.median_nanos
    }

    /// The interquartile range of round-trip latencies in nanoseconds.
    pub(crate) fn iqr_nanos(self) -> u128 {
        self.iqr_nanos
    }

    /// The summary of a dose whose samples are all exactly 1 nanosecond: median 1, IQR 0. Used to
    /// assemble a whole coherent [`DoseObservation`] fixture while [`Self::from_raw`] remains
    /// `todo!()` for the measurement milestone — the value is chosen to agree with that fixture's
    /// identical 1 ns samples, not to exercise any real derivation. Test-only.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        Self {
            median_nanos: 1,
            iqr_nanos: 0,
        }
    }
}
