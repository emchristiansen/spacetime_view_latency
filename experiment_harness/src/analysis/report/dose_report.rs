//! The report projection of one validated cumulative dose: raw latencies and retained evidence.

use serde::Serialize;

use crate::analysis::report::raw_latencies_report::RawLatenciesReport;
use crate::analysis::validate::trusted_dose::TrustedDose;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_summary::LatencySummary;

/// The report projection of one [`TrustedDose`] (spec: "Each dose report also retains its typed
/// coordinate/identity, logical cardinality, physical cardinalities, validated median and IQR, and event
/// evidence", and "embed every raw latency vector"). The fixed-shape, integer-exact trusted content types
/// are reused directly — no float and no lossy boundary is involved, so the report stays byte-faithful to
/// the trusted leaf: [`LatencySummary`] the exact R-1 median/IQR nanoseconds, [`PhysicalCardinalities`] the
/// family-typed row counts, [`EventEvidence`] the delivery counts, and [`DoseIndex`] the 1-based ladder
/// identity. The lossless raw vector is the one field with its own report projection —
/// [`RawLatenciesReport`], a fixed [`BATCH_SIZE_USIZE`](crate::params::BATCH_SIZE_USIZE) array with a manual
/// sequence `Serialize` — because the trusted type's own `Serialize` renders a variable-length sequence
/// that would not carry the frozen 1,000-sample cardinality in the report type.
#[derive(Debug, Serialize)]
pub(crate) struct DoseReport {
    /// The 1-based ladder index — the dose's typed coordinate identity.
    dose: DoseIndex,
    /// The cumulative logical x-axis count (`dose * BATCH_SIZE`) — the logical cardinality.
    logical_n: u64,
    /// The validated median/IQR summary, in exact integer nanoseconds.
    summary: LatencySummary,
    /// The lossless raw latency vector, embedded verbatim in exact integer nanoseconds.
    raw_latencies: RawLatenciesReport,
    /// Both physical table cardinalities, proven equal to the deterministic dataset expectation.
    physical_cardinalities: PhysicalCardinalities,
    /// The SDK logical delivery-event evidence (insert/delete/update counts and net delta).
    events: EventEvidence,
}

impl DoseReport {
    /// Project one validated dose's raw evidence. One input — the trusted dose — projected whole.
    pub(crate) fn of(dose: &TrustedDose) -> Self {
        // The fixed-shape integer-exact content types are reused directly (no lossy boundary); only the
        // lossless raw vector gets its own fixed-cardinality report projection.
        Self {
            dose: dose.dose(),
            logical_n: dose.logical_n(),
            summary: *dose.summary(),
            raw_latencies: RawLatenciesReport::of(dose.latencies()),
            physical_cardinalities: *dose.cardinalities(),
            events: *dose.events(),
        }
    }
}
