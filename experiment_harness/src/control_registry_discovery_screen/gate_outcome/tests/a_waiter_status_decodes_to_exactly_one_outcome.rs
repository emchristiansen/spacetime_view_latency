//! Every waiter termination decodes to exactly one outcome, and refusal is the only nonzero code
//! that is not inoperable.

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use crate::control_registry_discovery_screen::gate_outcome::{GateOutcome, GATE_REFUSAL_EXIT_CODE};

/// Coverage: the three classifications are disjoint and exhaustive over the shapes a termination can
/// take — success, the agreed refusal code, a completed exit carrying any other code, and a signal
/// carrying none. Admission and refusal are each a single exact code; everything else is inoperable,
/// so the unexpected statuses below deliberately share one arm rather than being separated.
///
/// The unexpected-nonzero case uses 1 deliberately: that is what Nushell's `error make` produces
/// for every parse, argument, `/proc`, clock and counter failure in the waiter, so it is the exact
/// status that used to be read as a refusal. If this test ever passes with 1 decoding to `Refused`,
/// the defect has come back.
///
/// The signal case is asserted through `code()` being `None` rather than by naming a signal number,
/// because it is the absent code, not the particular signal, that makes it undecidable as a verdict.
#[test]
fn a_waiter_status_decodes_to_exactly_one_outcome() {
    assert!(
        matches!(GateOutcome::of_status(exited(0)), GateOutcome::Admitted),
        "zero is the only admission"
    );
    assert!(
        matches!(
            GateOutcome::of_status(exited(GATE_REFUSAL_EXIT_CODE)),
            GateOutcome::Refused
        ),
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
fn exited(code: i32) -> ExitStatus {
    ExitStatus::from_raw(code << 8)
}

/// A process killed by `signal` before it could return a code.
fn signalled(signal: i32) -> ExitStatus {
    ExitStatus::from_raw(signal)
}
