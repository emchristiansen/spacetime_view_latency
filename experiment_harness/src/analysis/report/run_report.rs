//! The report projection of one validated run: its identity, provenance, and complete dose ladder.

use serde::Serialize;

use crate::analysis::report::dose_report::DoseReport;
use crate::analysis::report::run_provenance_report::RunProvenanceReport;
use crate::analysis::validate::trusted_run::TrustedRun;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::record_seq::RecordSeq;
use crate::params::NUM_DOSES_USIZE;

/// The report projection of one [`TrustedRun`]: its typed schedule coordinate, its collection-order
/// manifest sequence, its run-varying server/module provenance (spec: "Each `TrustedRun` owns its
/// run-varying database identity and server facts"), and its complete ten-dose ladder of raw observations.
/// The coordinate reuses the already-`Serialize` domain [`RunCoordinate`] directly; the ten doses are a
/// boxed fixed array so the dose cardinality is a property of the type.
#[derive(Debug, Serialize)]
pub(crate) struct RunReport {
    /// This run's schedule coordinate (cell, role, repetition block).
    coordinate: RunCoordinate,
    /// The campaign sequence of this run's manifest record — its position in the durable collection order.
    manifest_seq: RecordSeq,
    /// This run's run-varying database identity and server facts.
    provenance: RunProvenanceReport,
    /// The complete monotonic ten-dose ladder of raw observations, in canonical dose order.
    doses: Box<[DoseReport; NUM_DOSES_USIZE]>,
}

impl RunReport {
    /// Project one validated run's raw observations. Generic over the role marker `R` (role-agnostic:
    /// coordinate, manifest sequence, provenance, and doses do not depend on it), so one projection serves
    /// both the arm and control runs. One input — the trusted run — projected whole.
    pub(crate) fn of<R>(run: &TrustedRun<R>) -> Self {
        // Project the ten doses heap-first into the fixed ladder array; the coordinate reuses the domain
        // type directly and the provenance projects the run-varying facts.
        let doses: Vec<DoseReport> = run.doses().iter().map(DoseReport::of).collect();
        let doses = doses
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly NUM_DOSES_USIZE doses project into exactly NUM_DOSES_USIZE dose reports");
        Self {
            coordinate: run.coordinate().clone(),
            manifest_seq: run.manifest_seq(),
            provenance: RunProvenanceReport::of(run.provenance()),
            doses,
        }
    }
}
