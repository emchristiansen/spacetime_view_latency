//! One rung's complete, lossless evidence within an attempt.

use serde::Serialize;

use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;

/// The evidence produced by one rung of the frozen ladder.
///
/// This is the Pilot's progressive evidence unit: each rung is appended as it completes, so an
/// attempt that dies at rung four leaves four rungs of real evidence rather than nothing.
///
/// `global_rows` is derived in [`Self::observed`] from `rung` rather than passed in, so the recorded
/// ladder position and row count cannot disagree. The full [`RawLatencies`] is retained alongside
/// the [`LatencySummary`] so every reported summary stays reproducible from the record.
///
/// It carries no classification field — the spec forbids a Pilot attempt from authorizing a
/// performance conclusion, so there is deliberately nothing here to tempt one.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RungEvidence {
    rung: GlobalRowRung,
    global_rows: u64,
    subscribed_rows: u64,
    latencies: RawLatencies,
    summary: LatencySummary,
}

impl RungEvidence {
    /// Assemble one rung's evidence: its ladder position, the result-set size read back after the
    /// measured batch confirmed, and the sealed latency vector.
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

    /// The ladder position this evidence was observed at. Read by the sealing constructors of
    /// [`EvidenceArtifact`](super::evidence_artifact::EvidenceArtifact) and
    /// [`PartialEvidence`](super::partial_evidence::PartialEvidence) to prove their ordering
    /// invariants; the other fields are carried for the ledger, not read back in process.
    pub(crate) fn rung(&self) -> GlobalRowRung {
        self.rung
    }
}
