//! The outcome of a run's manifest-write transition.

use super::RunExecutionStopped;
use super::RunSeedingBackground;

/// What a manifest write yields: either the run advances into its pre-dose preparation phases
/// ([`RunSeedingBackground`], the first of the once-per-run warm-up → initial-set-check → dosing thread),
/// or it stopped short at the manifest ([`RunExecutionStopped`], the inert stopped-execution carrier its
/// linear cleanup owner then settles). There is no third state — a manifest write's contract either
/// returned success (yielding the warm-up phase) or it did not (yielding a stopped-execution carrier).
pub(crate) enum RunManifestStep {
    /// The manifest write's contract returned success; the run holds its receipt and enters the warm-up
    /// phase.
    Seeding(RunSeedingBackground),
    /// The manifest write failed; the run stopped short with a `WritingManifest` execution frontier.
    Stopped(RunExecutionStopped),
}
