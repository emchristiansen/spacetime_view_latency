//! Driver for the `ControlRegistry` discovery E3 screen.
//!
//! The frozen inventory, its execution order, gate totality, terminal-record settlement, and the
//! durable ledger seam are real here. The per-attempt acquisition — provision, publish, seed,
//! observe, time, validate, tear down — is [`todo!`] pending the acquisition milestone. Nothing in
//! this file measures anything yet.
//!
//! **Every fallible operation except ledger storage itself ends as exactly one terminal record.**
//! Ledger create, serialization, append, and flush are the deliberate exception: claiming durable
//! terminal coverage after the storage operation failed would be false, so those fail immediately.
//!
//! **There is no retry scheduler here, deliberately.** This invocation executes exactly the sixteen
//! frozen original identities, and each settles with exactly one terminal record.
//! `RetryEligibility::Retryable` on a `Provision` or `Connect` failure is a prospective fact
//! authorizing a separately frozen future retry inventory — never an instruction for this run to
//! loop. Scheduling retries here would need a bound the spec intentionally does not define, and
//! would recreate the dormant campaign's retry, reconciliation, and selection machinery that the
//! evidence lifecycle forbids extending. Re-running this screen is a new screen run, not a retry,
//! and mints no supersession link: every record this driver writes is an original.

use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, ensure, Context, Result};

use crate::control_registry_discovery_screen::attempt_inventory::AttemptInventory;
use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::retry_ordinal::RetryOrdinal;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::output_path::OutputPath;

/// Every identity this driver records is an original, so none supersedes an earlier attempt.
///
/// Spelled as a named constant at each call site rather than a bare `None`, because the absence of a
/// supersession link here is a frozen property of this screen — sixteen originals, no scheduler —
/// and not an argument that happened to be left empty.
const ORIGINAL_HAS_NO_SUPERSESSION: Option<RetryOrdinal> = None;

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

    let attempts = inventory.attempts();
    // Opened on the first attempt reached, never before: a run that freezes an inventory and then
    // refuses every slot must still leave a truthful ledger, but a run that fails to resolve its
    // waiter has not started and must leave the requested path untouched and reusable.
    let mut ledger: Option<File> = None;

    for (index, attempt) in attempts.iter().enumerate() {
        let attempt = *attempt;
        println!(
            "screen: {} at {} rows",
            attempt.target().canonical_tag(),
            attempt.rung().history_rows()
        );

        let ledger = match &mut ledger {
            Some(ledger) => ledger,
            empty => empty.insert(create_ledger(output)?),
        };

        let record = match admitted_by_gate(&host_waiter) {
            // Nothing was gated, and a waiter that cannot be spawned will not gate any later
            // attempt either. Measuring on an ungated host is what the gate exists to prevent, so
            // this slot and every remaining one settle here.
            Err(error) => {
                let reason = NotRunReason::GateInoperable {
                    diagnostic: DiagnosticArtifact::of_error(&error),
                };
                append_all(
                    ledger,
                    &settle_remaining(&attempts[index..], &pinned, seed, &reason)?,
                )?;
                return Err(error.context(
                    "the host gate could not be run, so this attempt and every remaining slot were \
                     recorded NotRun(GateInoperable) and the screen stopped",
                ));
            }
            // A refusal consumes no slot and does not abort the remaining slots: it is recorded and
            // the screen moves on, which is what makes the attempt inventory restart-safe.
            Ok(false) => ScreenRecord::not_run(
                attempt,
                &pinned,
                seed,
                ORIGINAL_HAS_NO_SUPERSESSION,
                NotRunReason::EnvironmentRefused,
            )?,
            Ok(true) => run_attempt(listen, module_wasm, attempt, &pinned, seed)?,
        };

        // Appended and flushed *before* the release verdict is acted on, so this attempt's own
        // evidence — including a release failure on the very last slot — is durable regardless of
        // what the verdict decides about the rest of the run.
        append_all(ledger, std::slice::from_ref(&record))?;

        if let Some(diagnostic) = record.release_failure_diagnostic() {
            let reason = NotRunReason::PriorAttemptReleaseFailed {
                diagnostic: diagnostic.clone(),
            };
            append_all(
                ledger,
                &settle_remaining(&attempts[index + 1..], &pinned, seed, &reason)?,
            )?;
            return Err(anyhow!(
                "attempt {} did not provably release every resource it acquired, so every \
                 remaining slot was recorded NotRun(PriorAttemptReleaseFailed) and the screen \
                 stopped: {diagnostic:?}",
                attempt.target().canonical_tag(),
            ));
        }
    }

    Ok(())
}

/// One attempt's terminal record, measured on its own fresh isolated instance.
///
/// The acquisition milestone fills this in: provision a verified pinned distribution and fresh
/// server/data directory, publish the hash-verified module, seed `K` controls with `N / K` history
/// rows each through the two atomic registry reducers only, take the `before` host observation
/// immediately before issuing the timed subscription, stop the clock inside `on_applied` as its
/// first statement, take the `after` observation immediately after the timed interval and before any
/// validation subscription or cache read, then subscribe the other three targets, validate all four
/// caches, settle, and tear down.
///
/// Returns a record rather than an error for every attempt-level failure: each of the nine failure
/// kinds settles into exactly one of the five record shapes, carrying the facts that existed at its
/// stage. Only a harness bug propagates.
fn run_attempt(
    _listen: ListenAddress,
    _module_wasm: &Path,
    _attempt: AttemptKey,
    _pinned: &PinnedArtifactIdentity,
    _seed: ScheduleSeed,
) -> Result<ScreenRecord> {
    todo!(
        "acquisition milestone: provision through the monotone PartialProvision prefix, publish, \
         seed through the atomic registry reducers, take the before observation, time the cold \
         apply inside on_applied, take the after observation, subscribe the validation targets, \
         read the four caches into a FourWayObservation, then seal ColdApplyEvidence or settle a \
         typed AttemptFailure into its stage's record shape, attempt disconnect and teardown, and \
         record the resulting ResourceDisposition"
    )
}

/// Settle every slot in `remaining` as `NotRun` under one reason.
///
/// Separated from the append so the settlement is a pure, testable function: the frozen inventory
/// promises one terminal record per predeclared attempt, and a run that stops early keeps that
/// promise only if the slots it never reached are recorded rather than omitted.
fn settle_remaining(
    remaining: &[AttemptKey],
    pinned: &PinnedArtifactIdentity,
    seed: ScheduleSeed,
    reason: &NotRunReason,
) -> Result<Vec<ScreenRecord>> {
    remaining
        .iter()
        .map(|attempt| {
            ScreenRecord::not_run(
                *attempt,
                pinned,
                seed,
                ORIGINAL_HAS_NO_SUPERSESSION,
                reason.clone(),
            )
        })
        .collect()
}

/// Append each record as one NDJSON line and flush.
///
/// Flushed per call: a screen killed partway keeps the attempts it finished, and the next attempt's
/// gate wait begins with this one already durable. A storage failure here is the one fallible
/// operation that propagates rather than settling into a record — a ledger cannot append a truthful
/// record about its own failure to append.
fn append_all(ledger: &mut File, records: &[ScreenRecord]) -> Result<()> {
    for record in records {
        let line = serde_json::to_string(record).context("encoding a screen record")?;
        writeln!(ledger, "{line}").context("appending a screen record")?;
    }
    ledger.flush().context("flushing the screen ledger")
}

/// Whether the host gate admits the next attempt.
///
/// Returns the refusal rather than raising it: the spec requires a refusal to record
/// `NotRun(EnvironmentRefused)` and leave the remaining slots runnable, so a non-zero exit is a
/// result here, not an error. A failure to *run* the waiter remains an error, because then nothing
/// was gated — and the caller settles that as `NotRun(GateInoperable)` for every remaining slot.
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

#[cfg(test)]
mod tests;
