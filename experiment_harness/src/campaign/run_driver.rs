//! The real effectful run driver's ownership boundary: own the connected run, settle through cleanup.

use crate::campaign::run_cleanup::RunCleanup;
use crate::campaign::run_cursor::RunSettled;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::plan::run::Run;

/// Linear owner of everything one provisioned, connected run needs to be driven to a cleanup-settled
/// terminal: the run's linear cleanup owner [`RunCleanup`] (the connected measured client plus the
/// provisioned resources), the run-cursor entry [`RunWritingManifest`] (which owns the observation sink),
/// the immutable [`ValidatedRunManifest`] (a value copy holding no live handle, for role identities and
/// the manifest record), and the [`Run`] (for dataset resolution and the subscribed target).
///
/// **Reserved shape, not yet an enforced boundary.** This type has no constructor and [`DriveRun`] has
/// no impl, so nothing constructs or drives it today; on its own it proves no compile-time property and
/// stands as documentation of the intended ownership plus the safety rationale below. The boundary
/// becomes enforced only once a constructor threads the live [`RunCleanup`] in from the provisioning
/// callback and [`DriveRun::drive`] is implemented to consume this owner into
/// `settle_executed`/`settle_stopped` on every branch.
///
/// **Why this is a struct and not a callable `todo!()` boundary.** A `fn(.., RunCleanup, ..) -> RunSettled`
/// whose body is `todo!()` is unsafe the instant it is *called*: the panic unwinds through the owned
/// [`RunCleanup`], dropping its `MustDisconnect` client guard — a loud take-on-consume "bomb" whose `Drop`
/// itself panics — and a panic during unwinding aborts the process. Modeling the boundary as an inert
/// ownership struct plus a declared-but-unimplemented [`DriveRun::drive`] means no callable path owns the
/// bomb until the real consuming implementation exists, so the skeleton can never be invoked into an
/// abort.
///
/// **The `RunCleanup` terminal-minter property is already enforced** (independently of this type).
/// [`RunCleanup`] is the sole minter of a run terminal: the only way to obtain a
/// [`RunSettled`]/[`RunIncomplete`] from it is [`RunCleanup::settle_executed`] /
/// [`RunCleanup::settle_stopped`], each performing the ordered disconnect-then-teardown while consuming the
/// owner. Neither [`RunCleanup`] nor its `resources: RunResources` has a `Drop` (that is deliberate — it is
/// what lets a consuming settle move their guards out without `unsafe`); the loud guards live on
/// `RunCleanup`'s `MustDisconnect` client and on
/// [`RunResources`](crate::provision::run_resources::RunResources)'s own members (the running server and
/// staged WASM), so abandoning a [`RunCleanup`] without settling still fires those guards rather than
/// silently leaking.
///
/// **Settlement boundary to be enforced by the driver** (future work — not yet in force). Once
/// implemented, [`DriveRun::drive`] will return [`RunSettled`] — never a `Result` whose `?` early-return
/// could skip cleanup — routing an effect failure through a consuming stop transition into
/// [`RunExecutionStopped`](crate::campaign::run_cursor::RunExecutionStopped) and then
/// [`RunCleanup::settle_stopped`], so that every branch consumes the owned [`RunCleanup`] into a settle.
/// Until that impl lands this is a documented obligation, not a compile-time guarantee: there is no
/// `drive` body yet, hence no branches to check.
pub(in crate::campaign) struct RunDriver {
    cleanup: RunCleanup,
    writing: RunWritingManifest,
    manifest: ValidatedRunManifest,
    run: Run,
}

/// The run-driving contract: consume the [`RunDriver`] and drive its run to a settled terminal.
///
/// Declared with **no implementation** until the measurement-milestone effect logic lands, so there is no
/// callable body that could panic-drop the owned cleanup bomb. The eventual implementation writes the
/// manifest record, applies the unmeasured warm-up slice and pre-dose check (the two `Run`-cursor
/// preparation phases), runs the ten measured doses with their post-write correctness/event checks, and
/// consumes the owned [`RunCleanup`] into `settle_executed`/`settle_stopped` on **every** branch —
/// returning [`RunSettled`] and never a `Result`.
trait DriveRun {
    /// Consume the driver, driving the run to a [`RunSettled`] terminal through the owned cleanup.
    fn drive(self) -> RunSettled;
}
