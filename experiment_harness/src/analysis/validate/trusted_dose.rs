//! The leaf of the trusted campaign graph: one validated cumulative dose observation.

use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;

/// One cumulative dose, proven complete and internally consistent by the single validation pass
/// ([`ValidatedCampaign::from_records`](super::validated_campaign::ValidatedCampaign::from_records)):
/// its raw sample count, exact R-1 summary recomputation, deterministic physical cardinalities, and
/// event-count identity all hold. The contents are the production trusted content types verbatim —
/// never their untrusted [`ingest`](crate::analysis::ingest) DTO mirrors — so a value that reaches this
/// leaf carries analysis-domain meaning, not merely wire syntax.
///
/// Fields are private with no defaults, and [`Self::new`] is `pub(super)`, so a `TrustedDose` can be
/// minted only from within the `validate` subtree (the validation pass) — never crate-wide. Downstream
/// analysis reaches the contents through read-only accessors, added when statistical code consumes them.
///
/// The role-dependent *expected* subscription/event evidence is not a separate stored field: the spec's
/// per-dose obligation is that the recorded [`EventEvidence`] (insert/delete/update counts and net
/// delta) and both [`PhysicalCardinalities`] equal the deterministic expectation derived from the run's
/// resolved dataset and dose batch (spec: "Derive each post-dose expected set cumulatively from typed
/// dataset evidence"). That cross-check is the validation pass's job against the single dataset
/// source-of-truth; storing a duplicate expected-set here would add drift surface, so this leaf retains
/// only the recorded evidence and the pass proves it matches.
pub(crate) struct TrustedDose {
    /// The 1-based ladder index, proven to appear exactly once per run across doses `1..=NUM_DOSES`.
    dose: DoseIndex,
    /// The cumulative logical x-axis count (`dose * BATCH_SIZE`), proven monotonic along the run ladder.
    logical_n: u64,
    /// The lossless raw latency vector, retained in full so every summary is exactly recomputable.
    latencies: RawLatencies,
    /// The median/IQR summary, proven to be the exact R-1 recomputation of `latencies`.
    summary: LatencySummary,
    /// Both physical table cardinalities, proven equal to the deterministic dataset expectation.
    cardinalities: PhysicalCardinalities,
    /// The SDK logical event evidence, proven to satisfy the insert/delete/net-delta identity.
    events: EventEvidence,
}

impl TrustedDose {
    /// Assemble a trusted dose from its already-cross-checked parts. The caller (the validation pass)
    /// owns proving every per-dose obligation; this constructor only binds the proven parts together so
    /// the private fields cannot be populated from outside the validation boundary.
    pub(super) fn new(
        dose: DoseIndex,
        logical_n: u64,
        latencies: RawLatencies,
        summary: LatencySummary,
        cardinalities: PhysicalCardinalities,
        events: EventEvidence,
    ) -> Self {
        Self {
            dose,
            logical_n,
            latencies,
            summary,
            cardinalities,
            events,
        }
    }

    /// The cumulative logical x-axis count (`dose * BATCH_SIZE`) of this dose — the preregistered ladder
    /// x-value the primary Theil–Sen estimator regresses each per-dose response against.
    pub(crate) fn logical_n(&self) -> u64 {
        self.logical_n
    }

    /// The R-1 median round-trip latency of this dose in nanoseconds — the per-dose response `L(N)` the
    /// classifier folds into each block's total change and into the frozen control-median margin.
    pub(crate) fn median_nanos(&self) -> u128 {
        self.summary.median_nanos()
    }

    /// The 1-based ladder index of this dose. Promoted to `pub(crate)` so the report can embed each dose's
    /// typed coordinate identity alongside its raw evidence (spec: "Each dose report also retains its typed
    /// coordinate/identity").
    pub(crate) fn dose(&self) -> DoseIndex {
        self.dose
    }

    /// The lossless raw latency vector of this dose — embedded verbatim in the authoritative report (spec:
    /// "embed every raw latency vector in the authoritative JSON"), so exact nanoseconds cross no lossy
    /// boundary.
    pub(crate) fn latencies(&self) -> &RawLatencies {
        &self.latencies
    }

    /// The validated median/IQR summary of this dose — the exact R-1 recomputation of [`Self::latencies`],
    /// projected into the report's per-dose evidence (spec: "its ... validated median and IQR").
    pub(crate) fn summary(&self) -> &LatencySummary {
        &self.summary
    }

    /// Both physical table cardinalities of this dose, proven equal to the deterministic dataset
    /// expectation — the report's per-dose physical evidence (spec: "physical cardinalities").
    pub(crate) fn cardinalities(&self) -> &PhysicalCardinalities {
        &self.cardinalities
    }

    /// The SDK logical event evidence of this dose (insert/delete/update counts and net delta) — the
    /// report's per-dose event evidence (spec: "event evidence").
    pub(crate) fn events(&self) -> &EventEvidence {
        &self.events
    }
}
