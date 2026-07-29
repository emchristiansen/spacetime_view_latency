//! Why a predeclared attempt was never executed.

use serde::Serialize;

/// The reasons a frozen slot can terminate without being run.
///
/// One variant, because the host gate is the only refusal this screen can currently produce: it runs
/// before the attempt provisions anything, so a refusal has measured nothing, consumes no slot, and
/// must not abort the remaining slots. A truthful `NotRun` inventory is itself evidence supporting
/// an indeterminate status, which is why a refused attempt is recorded rather than skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum NotRunReason {
    /// The host gate did not admit this attempt within its deadline.
    EnvironmentRefused,
}
