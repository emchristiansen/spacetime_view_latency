//! Which lifecycle stage a run had reached when it stopped short of completion.

use serde::Serialize;

use crate::dataset::dose_index::DoseIndex;

/// The stage a run occupied when an incompleteness was recorded. A run advances through these in
/// order — write its once-per-run manifest, apply the unmeasured background warm-up slice, check the
/// pre-dose expected set, apply each cumulative dose and persist its observation (writer contract
/// returning success), disconnect the measured client, tear down the isolated server — and a
/// [`RunFrontier`](super::run_frontier::RunFrontier) names exactly the one it stopped in.
///
/// The stages split by *what kind of step* stopped the run, never collapsing distinct steps under one
/// label. `WritingManifest` is the sink write of the manifest record; `BackgroundSeed` and
/// `InitialSetCheck` are the two pre-dose *server effects* that run after that record is written and
/// before the first dose (applying the pinned warm-up slice, then asserting the pre-dose result set);
/// `Dosing` covers a dose's measured writes, its post-write correctness/event checks, and its
/// observation sink write.
///
/// `Disconnecting` and `Teardown` are the two ordered cleanup stages the run's linear cleanup owner
/// ([`RunCleanup`](super::run_cleanup::RunCleanup)) runs after execution: a cleanup-stage
/// [`RunFrontier`](super::run_frontier::RunFrontier) names whichever one a real disconnect/teardown
/// failure occurred in ([`RunCleanupFailure::stage`](super::run_cleanup_failure::RunCleanupFailure::stage)
/// maps the failure to it). They name a stage a cleanup *call* failed in, not a claim of physical crash
/// persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum RunStage {
    /// Writing the run's once-per-run immutable manifest record, before any effect.
    WritingManifest,
    /// Applying the unmeasured pinned background warm-up slice through the normal insert reducers,
    /// after the manifest record and before any dose.
    BackgroundSeed,
    /// Asserting the pre-dose subscribed result set equals the seed-derived expected set, after the
    /// warm-up slice and before the first dose.
    InitialSetCheck,
    /// Applying the cumulative dose at the given ladder index — its measured writes, post-write
    /// correctness/event checks, and observation persistence (writer contract returning success).
    Dosing(DoseIndex),
    /// Disconnecting the measured subscriber after the last dose.
    Disconnecting,
    /// Tearing down the isolated server and owned paths after disconnect.
    Teardown,
}
