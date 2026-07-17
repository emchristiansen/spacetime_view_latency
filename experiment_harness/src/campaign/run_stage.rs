//! Which lifecycle stage a run had reached when it stopped short of completion.

use serde::Serialize;

use crate::dataset::dose_index::DoseIndex;

/// The stage a run occupied when an incompleteness was recorded. A run advances through these in
/// order — write its once-per-run manifest, apply each cumulative dose and persist its observation
/// (writer contract returning success), disconnect the measured client, tear down the isolated server —
/// and a
/// [`RunFrontier`](super::run_frontier::RunFrontier) names exactly the one it stopped in.
///
/// `Disconnecting` and `Teardown` are defined now as the structural positions after the dose ladder;
/// their effectful transitions belong to the future measurement driver. This Phase-1 skeleton reaches
/// them only through test-minted transition evidence, and that evidence proves only that the future
/// driver *reported* the transition — never that the external effect occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum RunStage {
    /// Writing the run's once-per-run immutable manifest record, before any dose.
    WritingManifest,
    /// Applying the cumulative dose at the given ladder index and persisting its observation (its
    /// writer contract returning success).
    Dosing(DoseIndex),
    /// Disconnecting the measured subscriber after the last dose (future effect).
    Disconnecting,
    /// Tearing down the isolated server and owned paths after disconnect (future effect).
    Teardown,
}
