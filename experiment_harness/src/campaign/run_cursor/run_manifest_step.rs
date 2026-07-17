//! The outcome of a run's manifest-write transition.

use super::RunDosing;
use super::RunIncomplete;

/// What a manifest write yields: either the run advances to its dose ladder ([`RunDosing`]), or it
/// stopped short at the manifest ([`RunIncomplete`]). There is no third state — a manifest write's
/// contract either returned success (yielding the dose ladder) or it did not (yielding a frontier).
pub(crate) enum RunManifestStep {
    /// The manifest write's contract returned success; the run holds its receipt and the dose ladder.
    Dosing(RunDosing),
    /// The manifest write failed; the run stopped short with a `WritingManifest` frontier.
    Incomplete(RunIncomplete),
}
