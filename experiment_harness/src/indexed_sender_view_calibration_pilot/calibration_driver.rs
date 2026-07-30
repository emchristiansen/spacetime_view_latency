//! Driver for the append-only E2 method-calibration pilot.
//!
//! **Phase 1 skeleton.** The frozen inventory, gate totality, ledger seam, and terminal settlement
//! are implemented; [`run_attempt`] is the sole `todo!()` and no measurement is runnable yet. Phase 2
//! must fill provisioning, publication, unrelated and own-slice seeding, the arm subscription, the
//! `before` observation, the paced append batch with its `on_insert` endpoint, the `after`
//! observation, the untimed witness subscription, row-by-row composition verification, typed
//! settlement, and teardown — without changing the frozen inventory.
//!
//! **Every attempt-level failure ends as exactly one terminal record.** There are two deliberate
//! exceptions, both of which abandon the run rather than misreport it. Ledger create, serialization,
//! append, and flush propagate, because claiming durable terminal coverage after the storage
//! operation failed would be false. And a *harness bug* — a record the type model refuses to build —
//! propagates out of [`run_attempt`], abandoning that attempt's slot and every later one unrecorded;
//! that is the accepted screen's contract too, and it is a defect in this harness rather than an
//! outcome of the experiment.
//!
//! **There is no retry scheduler here, and no retry identity to schedule.** This invocation executes
//! exactly the two frozen originals. `RetryEligibility::Retryable` on a `Provision` or `Connect`
//! failure is a prospective fact authorizing a separately frozen future inventory — which would be a
//! new vocabulary and a new SSOT authorization, not a loop here.

use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, ensure, Context, Error, Result};

use crate::indexed_sender_view_calibration_pilot::attempt_inventory::AttemptInventory;
use crate::indexed_sender_view_calibration_pilot::attempt_key::AttemptKey;
use crate::indexed_sender_view_calibration_pilot::calibration_record::CalibrationRecord;
use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::indexed_sender_view_calibration_pilot::gate_outcome::{
    GateOutcome, GATE_REFUSAL_EXIT_CODE, REFUSAL_EXIT_CODE_FLAG,
};
use crate::indexed_sender_view_calibration_pilot::not_run_reason::NotRunReason;
use crate::indexed_sender_view_calibration_pilot::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::manifest::listen_address::ListenAddress;
use crate::observation::output_path::OutputPath;

/// The gate every attempt waits behind, fixed rather than exposed as flags, so an attempt admitted
/// by a different gate cannot be compared to one admitted by this. Copied verbatim from the accepted
/// discovery screen, whose thresholds these already are.
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

/// Freeze the two predeclared attempts, then run each on its own fresh isolated instance behind the
/// host gate, appending exactly one terminal record per attempt.
///
/// The inventory is frozen *before* the waiter or the ledger is touched, so a design error fails
/// immediately rather than after the first eight-minute gate wait.
pub(crate) fn indexed_sender_view_calibration_pilot(
    listen: ListenAddress,
    module_wasm: &Path,
    host_waiter: &Path,
    output: &OutputPath,
    seed: u64,
) -> Result<()> {
    let inventory = AttemptInventory::frozen()?;
    // Resolved once, before anything is provisioned: a malformed pinned constant must fail at
    // startup rather than after the first attempt has already been measured against it.
    let pinned = PinnedArtifactIdentity::frozen()?;

    let host_waiter = resolve_host_waiter(host_waiter)?;
    ensure!(
        !output.path().exists(),
        "the calibration ledger {} already exists; evidence is never overwritten",
        output.path().display()
    );

    let attempts = inventory.attempts();
    // Opened on the first attempt reached, never before: a run that freezes an inventory and then
    // refuses every slot must still leave a truthful ledger, but a run that fails to resolve its
    // waiter has not started and must leave the requested path untouched and reusable.
    let mut ledger: Option<File> = None;

    for (index, attempt) in attempts.iter().enumerate() {
        let attempt = *attempt;
        println!("calibration: replicate {}", attempt.replicate().get());

        let ledger = match &mut ledger {
            Some(ledger) => ledger,
            empty => empty.insert(create_ledger(output)?),
        };

        let record = match gate_outcome(&host_waiter) {
            // The waiter could not be run at all.
            Err(error) => {
                return settle_gate_inoperable(ledger, &attempts[index..], &pinned, seed, error)
            }
            // The waiter ran but reached no verdict, which leaves this attempt exactly as ungated as
            // one whose waiter never started — so both settle through the same helper.
            Ok(GateOutcome::Inoperable { status }) => {
                return settle_gate_inoperable(
                    ledger,
                    &attempts[index..],
                    &pinned,
                    seed,
                    anyhow!(
                        "the host waiter {} terminated with {status} rather than admitting or \
                         refusing, so this attempt was never gated",
                        host_waiter.display(),
                    ),
                )
            }
            // A refusal consumes no measurement or retry budget and does not abort the remaining
            // slot. It does consume this replicate: the spec's decision rule needs both series, so a
            // refusal here is coverage loss the ledger states plainly rather than a neutral event.
            Ok(GateOutcome::Refused) => CalibrationRecord::not_run(
                attempt,
                &pinned,
                seed,
                NotRunReason::EnvironmentRefused,
            )?,
            Ok(GateOutcome::Admitted) => run_attempt(listen, module_wasm, attempt, &pinned, seed)?,
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
                "calibration replicate {} did not provably release every resource it acquired, so \
                 every remaining slot was recorded NotRun(PriorAttemptReleaseFailed) and the pilot \
                 stopped: {diagnostic:?}",
                attempt.replicate().get(),
            ));
        }
    }

    Ok(())
}

/// One attempt's terminal record, measured on its own fresh isolated instance.
///
/// **Phase 1: not implemented.** Phase 2 fills the acquisition and measurement lifecycle described
/// in this module's header. It must return a record rather than an error for every attempt-level
/// failure, so each of the ten failure kinds settles into exactly one of the five record shapes.
/// Only a harness bug — a record the type model refuses to build — propagates, and the caller's `?`
/// then abandons this slot and every later one without a record. That is the accepted screen's
/// contract verbatim; it is the one hole in per-attempt coverage, and it is deliberate.
fn run_attempt(
    _listen: ListenAddress,
    _module_wasm: &Path,
    _attempt: AttemptKey,
    _pinned: &PinnedArtifactIdentity,
    _seed: u64,
) -> Result<CalibrationRecord> {
    todo!("Phase 2: provision, seed, subscribe, bracket the paced append batch, verify, settle")
}

/// Settle every slot in `remaining` as `NotRun` under one reason.
///
/// Separated from the append so the settlement is a pure, testable function: the frozen inventory
/// promises one terminal record per predeclared attempt, and a run that stops early keeps that
/// promise only if the slots it never reached are recorded rather than omitted.
fn settle_remaining(
    remaining: &[AttemptKey],
    pinned: &PinnedArtifactIdentity,
    seed: u64,
    reason: &NotRunReason,
) -> Result<Vec<CalibrationRecord>> {
    remaining
        .iter()
        .map(|attempt| CalibrationRecord::not_run(*attempt, pinned, seed, reason.clone()))
        .collect()
}

/// Append each record as one NDJSON line and flush.
///
/// Flushed per call: a pilot killed partway keeps the attempts it finished. A storage failure here is
/// the one fallible operation that propagates rather than settling into a record — a ledger cannot
/// append a truthful record about its own failure to append.
fn append_all(ledger: &mut File, records: &[CalibrationRecord]) -> Result<()> {
    for record in records {
        let line = serde_json::to_string(record).context("encoding a calibration record")?;
        writeln!(ledger, "{line}").context("appending a calibration record")?;
    }
    ledger.flush().context("flushing the calibration ledger")
}

/// Record `remaining` — the current attempt and every slot after it — as `NotRun(GateInoperable)`
/// and stop the pilot.
///
/// Always returns `Err`, because there is no state in which it has anything to report: it exists for
/// the two ways an attempt can end up ungated, a waiter that could not be spawned and one that ran
/// without reaching a verdict.
fn settle_gate_inoperable(
    ledger: &mut File,
    remaining: &[AttemptKey],
    pinned: &PinnedArtifactIdentity,
    seed: u64,
    error: Error,
) -> Result<()> {
    let reason = NotRunReason::GateInoperable {
        diagnostic: DiagnosticArtifact::of_error(&error),
    };
    append_all(ledger, &settle_remaining(remaining, pinned, seed, &reason)?)?;
    Err(error.context(
        "the host gate did not gate this attempt, so it and every remaining slot were recorded \
         NotRun(GateInoperable) and the pilot stopped",
    ))
}

/// Run the host gate and decode what it concluded.
///
/// Returns a refusal rather than raising it: the spec requires a refusal to record
/// `NotRun(EnvironmentRefused)` and leave the remaining slot runnable, so a *verdict* of refusal is a
/// result here, not an error.
///
/// **Only the agreed exit code is that verdict.** The waiter has one deadline-refusal path and
/// seventeen paths that raise instead, and Nushell exits 1 for every one of them. Reading "nonzero"
/// as "refused" would file a host the gate never reached a verdict about under the reason that says
/// it was measured and found busy. [`GATE_REFUSAL_EXIT_CODE`] travels to the waiter as an argument so
/// the number has one definition.
fn gate_outcome(host_waiter: &Path) -> Result<GateOutcome> {
    let status = Command::new(host_waiter)
        .args(HOST_WAITER_ARGS)
        .arg(REFUSAL_EXIT_CODE_FLAG)
        .arg(GATE_REFUSAL_EXIT_CODE.to_string())
        .status()
        .with_context(|| format!("running the host waiter {}", host_waiter.display()))?;
    Ok(GateOutcome::of_status(status))
}

/// Create the ledger exclusively: this is the authority on the path being free, the earlier check
/// only being what makes an occupied path fail fast.
fn create_ledger(output: &OutputPath) -> Result<File> {
    File::options()
        .write(true)
        .create_new(true)
        .open(output.path())
        .with_context(|| {
            format!(
                "creating the calibration ledger {}",
                output.path().display()
            )
        })
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
