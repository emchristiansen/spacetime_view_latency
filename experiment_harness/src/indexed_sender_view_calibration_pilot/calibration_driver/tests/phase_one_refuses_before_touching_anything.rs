//! While Phase 2 is unwritten, invoking the pilot creates no ledger and runs no waiter.

use std::path::Path;

use tempfile::tempdir;

use crate::indexed_sender_view_calibration_pilot::calibration_driver::indexed_sender_view_calibration_pilot;
use crate::manifest::listen_address::ListenAddress;
use crate::observation::output_path::OutputPath;

/// Coverage: the Phase 1 guard fires before any side effect, and it is genuinely first.
///
/// Without the guard the wired subcommand would create the ledger, spawn the real host waiter and
/// block for up to its eight-minute budget, and only then panic at `run_attempt`'s `todo!()`. The
/// empty ledger would survive the panic, and the driver's own "evidence is never overwritten"
/// precondition would then refuse that path forever — so a run that measured nothing would
/// permanently consume the operator's requested output path.
///
/// The ordering is what makes this test meaningful, and it is established without any external
/// process: **both** paths handed in are deliberately nonexistent. If the guard were removed or moved
/// later, the driver would fail with a *different* error — canonicalising the missing waiter, or
/// creating the ledger — so asserting the Phase 1 message is asserting that nothing before it ran.
/// The ledger path is then checked to still not exist, which is the side effect that actually costs
/// the operator something.
///
/// The ledger lives under an RAII temporary directory rather than a fixed path in the system temp
/// dir, for two reasons that are about determinism rather than about the guard itself. A leftover
/// file from an earlier crashed run would trip the *precondition* assertion below and fail this test
/// for a reason unrelated to what it checks; and two concurrent runs sharing one fixed path would
/// race, since each asserts on a path the other may be creating or removing. A unique directory,
/// removed when it drops, makes both impossible.
#[test]
fn phase_one_refuses_before_touching_anything() {
    let listen = ListenAddress::parse("127.0.0.1:65000").expect("an explicit host:port parses");
    let module_wasm = Path::new("/nonexistent/experiment-module.wasm");
    let host_waiter = Path::new("/nonexistent/wait-for-free-ish-host.nu");

    let ledger_dir = tempdir().expect("a private temporary directory for the ledger");
    let ledger_path = ledger_dir.path().join("calibration.ndjson");
    let output = OutputPath::new(ledger_path.clone());

    assert!(
        !ledger_path.exists(),
        "the fixture ledger path must not exist before the call, or the assertion below proves \
         nothing"
    );

    let error = indexed_sender_view_calibration_pilot(listen, module_wasm, host_waiter, &output, 7)
        .expect_err("a Phase 1 skeleton must refuse to run");
    let rendered = format!("{error:#}");

    assert!(
        rendered.contains("Phase 1 type skeleton"),
        "the refusal must be the Phase 1 guard rather than a later failure, got {rendered:?}"
    );
    assert!(
        !ledger_path.exists(),
        "the guard must fire before the ledger is created; {} was left behind",
        ledger_path.display()
    );
}
