//! Untrusted mirror of [`LatencySummary`](crate::observation::latency_summary::LatencySummary).

use serde::Deserialize;

/// The wire form of a dose's median/IQR summary, both in nanoseconds. These are the *serialized*
/// summary values; `validate` recomputes the R-1 nearest-rank median and IQR from the lossless raw
/// latency vector and rejects any observation whose recorded summary disagrees, so the summary is
/// cross-checked against its own evidence rather than trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LatencySummaryDto {
    pub(crate) median_nanos: u128,
    pub(crate) iqr_nanos: u128,
}
