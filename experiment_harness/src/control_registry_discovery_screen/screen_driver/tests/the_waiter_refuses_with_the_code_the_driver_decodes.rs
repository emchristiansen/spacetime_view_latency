//! The real waiter's refusal exit is the exact code the driver decodes as a refusal, and nothing
//! else it can exit with is.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::control_registry_discovery_screen::gate_outcome::{
    GateOutcome, GATE_REFUSAL_EXIT_CODE, REFUSAL_EXIT_CODE_FLAG,
};

/// Arguments that force the deadline immediately, so the refusal path is reached in milliseconds
/// rather than the frozen eight-minute budget.
///
/// The failing condition is a memory floor above every value the waiter could report. It reads
/// `MemAvailable` in kB and divides by 1048576, and that field parses as `u64`, so the largest
/// available figure it can produce at all is `u64::MAX / 1048576` — about 1.76e13 GiB. A floor of
/// 1e15 GiB is two orders of magnitude beyond that, and still under 2^53, so it survives the
/// waiter's float parse exactly. The sample therefore cannot pass for any input the waiter is
/// capable of reading, not merely for any host that plausibly exists.
///
/// The bound has to come from the representable range rather than from physical plausibility, and a
/// low load ceiling would not give one at all: an idle host can genuinely report a one-minute load
/// under any positive threshold, which would let this test admit and then assert nothing.
const FORCE_REFUSAL_ARGS: [&str; 8] = [
    "--min-available-gib",
    "1000000000000000",
    "--samples",
    "1",
    "--interval-seconds",
    "0.01",
    "--timeout-minutes",
    "0",
];

/// An argument the waiter rejects before it samples anything, representative of the operational
/// paths that raise rather than reaching a verdict.
const INVALID_ARGS: [&str; 2] = ["--samples", "0"];

/// The status Nushell exits with when a script raises, which every `error make` in the waiter
/// produces and which an unflagged refusal must keep producing.
const NUSHELL_RAISE_STATUS: i32 = 1;

/// Coverage: the contract actually spans two processes, so it is checked against the real script
/// rather than a description of it. Both halves matter and neither implies the other — that a
/// refusal exits with the agreed code, and that a waiter which never measured the host does not.
///
/// The third case is the one that keeps this change honest. The visible-rows probe supplies no flag
/// and rejects every nonzero exit, so its pass/fail verdict would survive an unconditional exit 3
/// unchanged — what would not survive is *how* the refusal is reported. Without the flag the waiter
/// must still raise, at the status it has always raised at.
///
/// **What this pins is the status, not the wording.** The refusal message is checked by reading the
/// script: its interpolated text is preserved verbatim, now bound to `refusal` so both branches
/// share one value, and the unflagged branch still passes that same value to `error make`. What is
/// preserved is the text, not where it is built. Asserting the wording here would mean capturing and
/// matching Nushell's rendered error output, whose framing is the shell's to change; there is no
/// output-capture pattern in this crate's tests to follow for it, and inventing one would pin the
/// renderer rather than the script.
///
/// Sub-second by construction: nothing is provisioned, no server starts, and the deadline is already
/// past on the first sample.
#[test]
fn the_waiter_refuses_with_the_code_the_driver_decodes() {
    let refused = run(&FORCE_REFUSAL_ARGS, WithRefusalCode::Yes);
    assert!(
        matches!(GateOutcome::of_status(refused), GateOutcome::Refused),
        "a completed deadline refusal must decode as a verdict, got {refused}"
    );
    assert_eq!(
        refused.code(),
        Some(GATE_REFUSAL_EXIT_CODE),
        "the waiter must refuse with exactly the code the driver was told to expect"
    );

    let invalid = run(&INVALID_ARGS, WithRefusalCode::Yes);
    assert!(
        matches!(
            GateOutcome::of_status(invalid),
            GateOutcome::Inoperable { .. }
        ),
        "a waiter that rejected its arguments measured nothing and must stop the run, got {invalid}"
    );
    assert_ne!(
        invalid.code(),
        Some(GATE_REFUSAL_EXIT_CODE),
        "an operational failure must never be reportable as an environmental refusal"
    );

    let unflagged = run(&FORCE_REFUSAL_ARGS, WithRefusalCode::No);
    assert_eq!(
        unflagged.code(),
        Some(NUSHELL_RAISE_STATUS),
        "without the flag a refusal must still raise at the status it always raised at, which is \
         what the visible-rows probe's accepted evidence was collected against"
    );
    assert!(
        matches!(
            GateOutcome::of_status(unflagged),
            GateOutcome::Inoperable { .. }
        ),
        "and an unflagged refusal must not be readable as this screen's refusal verdict, got \
         {unflagged}"
    );
}

/// Whether this invocation opts into the typed refusal contract.
enum WithRefusalCode {
    Yes,
    No,
}

/// Run the real waiter, silencing its per-sample chatter, and return how it terminated.
fn run(args: &[&str], refusal_code: WithRefusalCode) -> std::process::ExitStatus {
    let mut command = Command::new(waiter());
    command
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let WithRefusalCode::Yes = refusal_code {
        command
            .arg(REFUSAL_EXIT_CODE_FLAG)
            .arg(GATE_REFUSAL_EXIT_CODE.to_string());
    }
    command
        .status()
        .expect("the host waiter is a tracked executable in this repository")
}

/// The waiter as the driver resolves it: the tracked script, addressed from this crate rather than
/// from wherever the test happens to be run.
fn waiter() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("scripts")
        .join("wait-for-free-ish-host.nu")
}
