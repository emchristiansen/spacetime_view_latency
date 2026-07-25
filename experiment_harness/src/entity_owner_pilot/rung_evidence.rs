//! One rung's complete, lossless evidence within an attempt.

use serde::Serialize;

use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;

/// The evidence produced by one rung of the frozen `N_global` ladder: where on the ladder it sat,
/// how large the observed result set was there, the lossless per-write latency vector, and its
/// derived summary.
///
/// This is the Pilot's *progressive* evidence unit — the Parked Frontier requires the durable ledger
/// to carry "progressive rung evidence", so each rung is appended as it completes rather than held
/// until the attempt finishes. An attempt that dies at rung four therefore leaves four rungs of real
/// evidence behind instead of nothing.
///
/// `global_rows` is derived in [`Self::observed`] from `rung` rather than passed in, so the recorded
/// ladder position and the recorded row count are single-sourced and cannot disagree. Retaining the
/// full [`RawLatencies`] alongside the [`LatencySummary`] is what makes the record "sufficient to
/// reproduce every reported summary" — the summary is a convenience, never the only copy.
///
/// It carries no classification. The spec forbids a Pilot attempt from authorizing a performance
/// conclusion, so there is deliberately no shape, verdict, or comparison field here to tempt one.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RungEvidence {
    rung: GlobalRowRung,
    global_rows: u64,
    subscribed_rows: u64,
    latencies: RawLatencies,
    summary: LatencySummary,
}

impl RungEvidence {
    /// Assemble one rung's evidence from what was actually observed at it: the ladder position, the
    /// result-set size read back after the measured batch confirmed, and the sealed latency vector.
    ///
    /// The summary is derived here through the same [`LatencySummary::from_raw`] the historical
    /// campaign uses, so Pilot and campaign summaries are computed by one implementation.
    pub(crate) fn observed(
        rung: GlobalRowRung,
        subscribed_rows: u64,
        latencies: RawLatencies,
    ) -> Self {
        let summary = LatencySummary::from_raw(&latencies);
        Self {
            rung,
            global_rows: rung.global_rows(),
            subscribed_rows,
            latencies,
            summary,
        }
    }

    /// The ladder position this evidence was observed at.
    pub(crate) fn rung(&self) -> GlobalRowRung {
        self.rung
    }

    /// The number of rows the measured subscriber's subscription actually returned at this rung.
    pub(crate) fn subscribed_rows(&self) -> u64 {
        self.subscribed_rows
    }
}
