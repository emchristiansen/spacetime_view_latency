//! The report projection of one validated cumulative dose: raw latencies and retained evidence.

use serde::Serialize;

use crate::analysis::report::raw_latencies_report::RawLatenciesReport;
use crate::analysis::validate::trusted_dose::TrustedDose;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_summary::LatencySummary;
use crate::plan::growth_regime::GrowthRegime;

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
///
/// Reports three distinct cardinality fields (spec: "Report three distinct cardinality fields per
/// dose"): [`Self::n_driving`] is the sole x-axis every estimator consumes; [`Self::n_total`] and
/// [`Self::n_own`] are additive auditability fields, never substituted for `n_driving` in a fit.
#[derive(Debug, Serialize)]
pub(crate) struct DoseReport {
    /// The 1-based ladder index — the dose's typed coordinate identity.
    dose: DoseIndex,
    /// The cumulative driving-role row count (`dose * BATCH_SIZE`) — the sole x-axis every Theil–Sen
    /// estimator, `T_block` classifier, and secondary β fit consumes (spec: "`n_driving` is the only
    /// x-axis consumed by every ... fit").
    n_driving: u64,
    /// `n_driving` plus the regime-appropriate pinned logical baseline (spec: "`n_total` is `n_driving`
    /// plus the regime-appropriate pinned logical baseline"). Reported for auditability only; never an
    /// estimator x-axis, since the additive offset is not invariant under `L(N) = a + bN^β`.
    n_total: u64,
    /// The measured-role result-slice size: the pinned constant under `UnrelatedGrowth`, `n_driving`
    /// under `OwnSliceGrowth` (spec: "`n_own` is the measured-role result-slice size"). Reported for
    /// auditability only; never an estimator x-axis.
    n_own: u64,
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
    /// Project one validated dose's raw evidence, plus its run's growth regime (needed to derive the
    /// regime-appropriate pinned baseline for `n_total`/`n_own`). The trusted dose alone supplies
    /// `n_driving`; the two additive auditability fields are derived here and never fed back into any
    /// estimator.
    pub(crate) fn of(dose: &TrustedDose, growth_regime: GrowthRegime) -> Self {
        // The fixed-shape integer-exact content types are reused directly (no lossy boundary); only the
        // lossless raw vector gets its own fixed-cardinality report projection.
        let n_driving = dose.logical_n();
        let pinned_baseline_rows = growth_regime.pinned_baseline_rows();
        let n_total = pinned_baseline_rows
            .checked_add(n_driving)
            .expect("dataset logical-row count must not overflow u64");
        let n_own = match growth_regime {
            GrowthRegime::UnrelatedGrowth => pinned_baseline_rows,
            GrowthRegime::OwnSliceGrowth => n_driving,
        };
        Self {
            dose: dose.dose(),
            n_driving,
            n_total,
            n_own,
            summary: *dose.summary(),
            raw_latencies: RawLatenciesReport::of(dose.latencies()),
            physical_cardinalities: *dose.cardinalities(),
            events: *dose.events(),
        }
    }
}
