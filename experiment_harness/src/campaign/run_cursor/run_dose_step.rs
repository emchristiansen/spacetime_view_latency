//! The outcome of drawing the next dose from a run's ladder.

use super::RunAwaitingDose;
use super::RunExecuted;

/// What drawing the next dose yields: either a dose remains and the run awaits its observation
/// ([`RunAwaitingDose`]), or the ladder is drained and the run reaches its inert exhausted-execution
/// carrier ([`RunExecuted`]), which its linear cleanup owner then settles. Exhaustion of the fixed ladder
/// iterator — not a count — is what moves the run off the dose ladder.
pub(crate) enum RunDoseStep {
    /// A dose remains; the run awaits that dose's observation.
    Awaiting(RunAwaitingDose),
    /// The dose ladder is drained; the run reached its exhausted-execution carrier.
    Exhausted(RunExecuted),
}
