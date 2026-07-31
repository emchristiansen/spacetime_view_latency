//! Every waiter termination decodes to exactly one outcome, and refusal is the only nonzero code
//! that is not inoperable.

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use crate::indexed_sender_view_calibration_pilot::gate_outcome::{
    GateOutcome, GATE_REFUSAL_EXIT_CODE,
};

/// Coverage: the three classifications are disjoint and exhaustive over the shapes a termination can
/// take — success, the agreed refusal code, a completed exit carrying any other code, and a signal
/// carrying none.
///
/// This decoder is copied from the accepted screen, where the original defect was reading *any*
/// nonzero status as a refusal. That mattered because the two are settled differently and
/// irreversibly: a refusal records `NotRun(EnvironmentRefused)` and leaves the remaining slot
/// runnable, while an inoperable gate settles the whole remaining suffix and stops the run. Filing an
/// unreachable `/proc` as "the host was measured and found busy" writes a claim the run never
/// established.
///
/// The unexpected-nonzero case uses 1 deliberately: that is what Nushell's `error make` produces for
/// every parse, argument, `/proc`, clock and counter failure in the waiter, so it is the exact status
/// that used to be misread. If this test ever passes with 1 decoding to `Refused`, the defect is back.
///
/// The signal case is asserted through `code()` being `None` rather than by naming a signal number,
/// because it is the absent code, not the particular signal, that makes it undecidable as a verdict.
#[test]
fn a_waiter_status_decodes_to_exactly_one_outcome() {
    assert!(
        matches!(GateOutcome::of_status(exited(0)), GateOutcome::Admitted),
        "zero is the only admission"
    );
    let refusal = u8::try_from(GATE_REFUSAL_EXIT_CODE)
        .expect("the agreed refusal code is a process exit code, so it fits in a byte");
    assert!(
        matches!(GateOutcome::of_status(exited(refusal)), GateOutcome::Refused),
        "the agreed refusal code is the only verdict-bearing nonzero exit"
    );

    // Representative completed statuses other than admission and the agreed refusal, plus a signal.
    // A code is 8 bits, so these stand for the rest rather than enumerating it: 1 is what the waiter
    // actually raises, 2 is conventional argument misuse, and 255 is the top of the range.
    for status in [exited(1), exited(2), exited(255), signalled(9)] {
        assert!(
            matches!(
                GateOutcome::of_status(status),
                GateOutcome::Inoperable { .. }
            ),
            "a status that carries no verdict must stop the run, got {status}"
        );
    }

    assert!(
        signalled(9).code().is_none(),
        "a signalled termination reports no code, which is why it cannot be a verdict"
    );
    assert_ne!(
        GATE_REFUSAL_EXIT_CODE, 1,
        "the refusal code must never collide with Nushell's raise status, or an inoperable waiter \
         would be recorded as a refusal again"
    );
}

/// A process that ran to completion and returned `code`.
///
/// The parameter is a `u8` because that is exactly the range a process exit code occupies, and the
/// wait-status encoding places it in the second byte. Taking an `i32` and relying on `checked_mul`
/// would not have been the same guarantee: `checked_mul` rejects only i32 overflow, so 256 or -1
/// would have sailed through and silently produced a status for a *different* code — or for a
/// signal. Narrowing the type rules that out at the call site instead of detecting it afterwards.
fn exited(code: u8) -> ExitStatus {
    let raw = i32::from(code)
        .checked_mul(256)
        .expect("a byte-sized exit code cannot overflow i32 when shifted into the wait status");
    ExitStatus::from_raw(raw)
}

/// A process killed by `signal` before it could return a code.
fn signalled(signal: i32) -> ExitStatus {
    ExitStatus::from_raw(signal)
}
