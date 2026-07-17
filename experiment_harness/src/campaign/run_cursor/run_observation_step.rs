//! The outcome of a run's dose-observation write transition.

use super::RunDosing;
use super::RunIncomplete;

/// What writing a dose's observation yields: either the run returns to its between-doses state to draw
/// the next dose ([`RunDosing`]), or it stopped short at this dose ([`RunIncomplete`]). There is no
/// third state — the observation write's contract either returned success or it did not.
pub(crate) enum RunObservationStep {
    /// The observation write's contract returned success; the run may draw its next dose.
    Dosing(RunDosing),
    /// The observation write failed; the run stopped short with a `Dosing` frontier.
    Incomplete(RunIncomplete),
}
