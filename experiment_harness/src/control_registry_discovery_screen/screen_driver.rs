//! Phase 1 driver skeleton for the `ControlRegistry` discovery E3 screen.
//!
//! The frozen inventory, its execution order, the gate/`NotRun` path, and the durable ledger seam
//! are real here, because those are what the freeze constrains and what the focused tests prove.
//! The per-attempt acquisition — provision, publish, seed, subscribe, time, validate, tear down —
//! is [`todo!`] pending Phase 2. Nothing in this file measures anything yet, and no evidence line
//! this skeleton could write would be valid.

use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};

use crate::control_registry_discovery_screen::attempt_inventory::AttemptInventory;
use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::method_facts::MethodFacts;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::screen_composition::ScreenComposition;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::control_registry_discovery_screen::supersession::Supersession;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::output_path::OutputPath;

/// The gate every attempt waits behind, fixed rather than exposed as flags, so an attempt admitted
/// by a different gate cannot be compared to one admitted by this. Copied verbatim from
/// [`crate::entity_owner_visible_rows_probe`], whose thresholds these already are.
const HOST_WAITER_ARGS: [&str; 12] = [
    "--load-per-cpu",
    "1.5",
    "--min-available-gib",
    "2",
    "--max-swap-io-mib-per-sec",
    "16",
    "--interval-seconds",
    "15",
    "--samples",
    "2",
    "--timeout-minutes",
    "8",
];

/// Freeze the sixteen predeclared attempts, then run each on its own fresh isolated instance behind
/// the host gate, appending exactly one terminal record per attempt.
///
/// The inventory is frozen *before* the waiter or the ledger is touched, so a design error fails
/// immediately rather than after the first eight-minute gate wait.
pub(crate) fn control_registry_discovery_screen(
    listen: ListenAddress,
    module_wasm: &Path,
    host_waiter: &Path,
    output: &OutputPath,
    seed: ScheduleSeed,
) -> Result<()> {
    let inventory = AttemptInventory::frozen(seed)?;
    // Resolved once, before anything is provisioned: a malformed pinned constant must fail at
    // startup rather than after the first attempt has already been measured against it.
    let pinned = PinnedArtifactIdentity::frozen()?;

    let host_waiter = resolve_host_waiter(host_waiter)?;
    ensure!(
        !output.path().exists(),
        "the screen ledger {} already exists; evidence is never overwritten",
        output.path().display()
    );

    // Opened on the first attempt reached, never before: a run that freezes an inventory and then
    // refuses every slot must still leave a truthful ledger, but a run that fails to resolve its
    // waiter has not started and must leave the requested path untouched and reusable.
    let mut ledger: Option<File> = None;
    for attempt in inventory.attempts() {
        let target = attempt.target();
        println!(
            "screen: {} at {} rows",
            target.canonical_tag(),
            attempt.rung().history_rows()
        );

        let ledger = match &mut ledger {
            Some(ledger) => ledger,
            empty => empty.insert(create_ledger(output)?),
        };

        // A refusal consumes no slot and does not abort the remaining slots: it is recorded and the
        // screen moves on, which is what makes the attempt inventory restart-safe.
        let record = match admitted_by_gate(&host_waiter)? {
            false => not_run_record(*attempt, &pinned, seed, NotRunReason::EnvironmentRefused)?,
            true => run_attempt(listen, module_wasm, *attempt, &pinned, seed)?,
        };

        let line = serde_json::to_string(&record).context("encoding a screen record")?;
        writeln!(ledger, "{line}").context("appending a screen record")?;
        // Flushed per attempt: a screen killed partway keeps the attempts it finished, and the next
        // attempt's gate wait begins with this one already durable.
        ledger.flush().context("flushing a screen record")?;
    }
    Ok(())
}

/// One attempt's terminal record, measured on its own fresh isolated instance.
///
/// Phase 2 fills this in: provision a verified pinned distribution and fresh server/data directory,
/// publish the hash-verified module, seed `K` controls with `N / K` history rows each through the
/// two atomic registry reducers only, observe the host, subscribe to the timed target and stop
/// inside `on_applied` as its first statement, observe the host again, then validate all four caches
/// and tear down.
fn run_attempt(
    _listen: ListenAddress,
    _module_wasm: &Path,
    _attempt: AttemptKey,
    _pinned: &PinnedArtifactIdentity,
    _seed: ScheduleSeed,
) -> Result<ScreenRecord> {
    todo!(
        "Phase 2: provision, publish, seed through the atomic registry reducers, observe the host, \
         time the cold apply inside on_applied, observe the host again, read all four caches into a \
         FourWayObservation, then seal ColdApplyEvidence or record a typed AttemptFailure, and tear \
         down"
    )
}

/// The record for a slot the gate never admitted.
///
/// Carries no provision provenance and no host observations, because the gate runs before anything
/// is provisioned and a refused attempt genuinely observed nothing — the `NotRun` variant has no
/// fields for them to be absent from. It still carries identity, composition, frozen method facts,
/// and the pinned artifact identity, so a reader can see what this slot was going to measure.
fn not_run_record(
    attempt: AttemptKey,
    pinned: &PinnedArtifactIdentity,
    seed: ScheduleSeed,
    reason: NotRunReason,
) -> Result<ScreenRecord> {
    Ok(ScreenRecord::NotRun {
        key: attempt,
        composition: ScreenComposition::of(attempt),
        method: MethodFacts::frozen(),
        pinned: pinned.clone(),
        supersession: Supersession::of(attempt.retry(), None)?,
        schedule_seed: seed.get(),
        reason,
    })
}

/// Whether the host gate admits the next attempt.
///
/// Returns the refusal rather than raising it: the spec requires a refusal to record
/// `NotRun(EnvironmentRefused)` and leave the remaining slots runnable, so a non-zero exit is a
/// result here, not an error. A failure to *run* the waiter at all remains an error, because then
/// nothing was gated.
fn admitted_by_gate(host_waiter: &Path) -> Result<bool> {
    let status = Command::new(host_waiter)
        .args(HOST_WAITER_ARGS)
        .status()
        .with_context(|| format!("running the host waiter {}", host_waiter.display()))?;
    Ok(status.success())
}

/// Create the ledger exclusively, as the Pilot's and the probe's are: this is the authority on the
/// path being free, the earlier check only being what makes an occupied path fail fast.
fn create_ledger(output: &OutputPath) -> Result<File> {
    File::options()
        .write(true)
        .create_new(true)
        .open(output.path())
        .with_context(|| format!("creating the screen ledger {}", output.path().display()))
}

/// Resolve the waiter to a canonical executable file, so it is invoked as a program and never
/// through a shell.
fn resolve_host_waiter(host_waiter: &Path) -> Result<PathBuf> {
    let resolved = fs::canonicalize(host_waiter)
        .with_context(|| format!("resolving the host waiter {}", host_waiter.display()))?;
    let metadata = fs::metadata(&resolved)
        .with_context(|| format!("stat-ing the host waiter {}", resolved.display()))?;
    ensure!(
        metadata.is_file(),
        "the host waiter {} is not a regular file",
        resolved.display()
    );
    ensure!(
        metadata.permissions().mode() & 0o111 != 0,
        "the host waiter {} is not executable",
        resolved.display()
    );
    Ok(resolved)
}
