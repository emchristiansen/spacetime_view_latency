//! What one run of the host gate concluded.

use std::process::ExitStatus;

/// The exit status the waiter uses for a completed deadline refusal, and nothing else.
///
/// Three because the statuses below it are all spoken for: zero is admission, one is what Nushell
/// raises from `error make`, and two is the conventional argument-misuse status.
///
/// **Defined here and passed to the waiter as an argument**, rather than written into the script and
/// re-declared here. The decoder is the only party that has to know the number, so it is the only
/// party that names it; a copy in the script would be free to drift from this one, silently, and
/// drift in exactly this value silently reclassifies evidence.
pub(crate) const GATE_REFUSAL_EXIT_CODE: i32 = 3;

/// The waiter flag that carries [`GATE_REFUSAL_EXIT_CODE`].
///
/// Supplying it is what opts this caller into the typed contract; a caller that supplies neither
/// keeps the waiter's historical raise-on-refusal behaviour.
pub(crate) const REFUSAL_EXIT_CODE_FLAG: &str = "--refusal-exit-code";

/// The three things running the gate can mean, and the only three.
///
/// The distinction that matters is not pass/fail but **whether the gate reached a verdict**. A
/// refusal is a verdict: the waiter sampled the host for its whole budget and concluded it was too
/// contended, which terminally settles one slot and leaves the rest of the inventory runnable. An
/// inoperable gate reached no verdict — malformed `/proc`, an unreadable page size, a clock that did
/// not advance, a rejected argument, a signal — and the run stops there.
///
/// **Stopping is not a prediction that the fault is permanent.** It is that this invocation can no
/// longer demonstrate that what it measures was gated, and continuing past a gate whose integrity is
/// gone would produce records claiming a guarantee the run cannot support.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GateOutcome {
    /// The waiter observed the required consecutive passing samples.
    Admitted,
    /// The waiter reached its deadline and refused. A verdict, not a malfunction.
    Refused,
    /// The waiter terminated without reaching a verdict, so nothing was gated.
    Inoperable { status: ExitStatus },
}

impl GateOutcome {
    /// Decode one waiter termination.
    ///
    /// `status.code()` is `None` exactly when a signal killed the process, which is inoperable for
    /// the same reason an unexpected code is: no verdict was reached. Both land in the catch-all
    /// rather than being enumerated, so a status this decoder has never seen can only ever be read
    /// as "the gate did not answer" — the safe direction, and the one that stops the run.
    pub(crate) fn of_status(status: ExitStatus) -> Self {
        match status.code() {
            Some(0) => Self::Admitted,
            Some(GATE_REFUSAL_EXIT_CODE) => Self::Refused,
            _ => Self::Inoperable { status },
        }
    }
}
