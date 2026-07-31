//! Driver for the append-only E2 method-calibration pilot.
//!
//! The frozen inventory, its execution order, gate totality, per-attempt acquisition, and the
//! durable ledger seam all live here. Each attempt provisions its own pinned instance, publishes the
//! hash-verified module, refuses an aliased measured identity before writing anything, seeds the
//! unrelated population and the subscriber's own slice one confirmed single-row transaction at a
//! time, subscribes the measured arm, brackets a paced append batch with host observations,
//! subscribes the untimed composition witness, validates both caches row by row, and tears down.
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
use std::time::Instant;

use anyhow::{anyhow, ensure, Context, Error, Result};
use spacetimedb_sdk::{Identity, Timestamp};

use crate::client::connected_client::ConnectedClient;
use crate::client::paced_append::PacedAppend;
use crate::entity_owner_pilot::attempt_provenance::{
    AttemptProvenance, DistributionFacts, ServerFacts,
};
use crate::indexed_sender_view_calibration_pilot::attempt_failure::AttemptFailure;
use crate::indexed_sender_view_calibration_pilot::attempt_inventory::AttemptInventory;
use crate::indexed_sender_view_calibration_pilot::attempt_key::AttemptKey;
use crate::indexed_sender_view_calibration_pilot::attempted_outcome::AttemptedOutcome;
use crate::indexed_sender_view_calibration_pilot::calibration_expectation::{
    ensure_measured_is_not_the_unrelated_owner, unrelated_owner, CalibrationExpectation,
};
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, MAX_PACED_SAMPLES_USIZE, OWN_ACTIVITY_ID_BASE, OWN_CONTROL_UUID,
    SUBSCRIBER_OWN_ROWS, UNRELATED_ACTIVITY_ID_BASE, UNRELATED_CONTROL_UUID,
};
use crate::indexed_sender_view_calibration_pilot::calibration_record::CalibrationRecord;
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;
use crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries;
use crate::indexed_sender_view_calibration_pilot::calibration_target::CalibrationTarget;
use crate::indexed_sender_view_calibration_pilot::composition_mismatch::CompositionMismatch;
use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::indexed_sender_view_calibration_pilot::expected_population::expected_ts_micros;
use crate::indexed_sender_view_calibration_pilot::failure_kind::FailureKind;
use crate::indexed_sender_view_calibration_pilot::gate_outcome::{
    GateOutcome, GATE_REFUSAL_EXIT_CODE, REFUSAL_EXIT_CODE_FLAG,
};
use crate::indexed_sender_view_calibration_pilot::host_observations::HostObservations;
use crate::indexed_sender_view_calibration_pilot::not_run_reason::NotRunReason;
use crate::indexed_sender_view_calibration_pilot::observed_row::ObservedRow;
use crate::indexed_sender_view_calibration_pilot::paced_sample_nanos::PacedSampleNanos;
use crate::indexed_sender_view_calibration_pilot::partial_evidence::PartialEvidence;
use crate::indexed_sender_view_calibration_pilot::partial_provision::PartialProvision;
use crate::indexed_sender_view_calibration_pilot::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::indexed_sender_view_calibration_pilot::rejected_series::RejectedSeries;
use crate::indexed_sender_view_calibration_pilot::resource_disposition::ResourceDisposition;
use crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::bindings::{IndexedControlActivity, SubscriptionHandle};
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::observation::latency_sample::LatencySample;
use crate::observation::output_path::OutputPath;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;
use crate::view_read_set_campaign::campaign_driver::observe_environment;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

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
            ),
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
                &settle_remaining(&attempts[index + 1..], &pinned, seed, &reason),
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
/// Provisions a verified pinned distribution and fresh server/data directory, publishes the
/// hash-verified module, connects, refuses an aliased measured identity before any write, seeds both
/// populations, subscribes the measured arm, takes the `before` host observation immediately before
/// the paced append batch, stops each sample's clock inside the arm's `on_insert` as its first
/// statement, takes the `after` observation immediately after the batch returns and before the
/// witness subscription or any cache read, then subscribes the witness, validates both caches row by
/// row, settles, and tears down.
///
/// **Returns a record rather than an error for every attempt-level failure.** Each of the twelve
/// failure kinds settles into exactly one of the five record shapes, carrying the facts that existed
/// at its stage and no others. Only a harness bug — a record the type model refuses to build —
/// propagates.
///
/// **Teardown is ordered ahead of the harness-bug path deliberately.** The driven outcome is bound
/// but not unwrapped until `resources.teardown()` has run, because `RunResources` and both its
/// members assert on `Drop` that they were handed off explicitly; propagating first would abort
/// through those assertions and lose the real error behind a leak panic.
fn run_attempt(
    listen: ListenAddress,
    module_wasm: &Path,
    attempt: AttemptKey,
    pinned: &PinnedArtifactIdentity,
    seed: u64,
) -> Result<CalibrationRecord> {
    let published = match provision(listen, module_wasm) {
        Ok(published) => published,
        Err(failure) => {
            return CalibrationRecord::not_provisioned(
                attempt,
                pinned,
                seed,
                failure.partial,
                AttemptFailure::observed(
                    FailureKind::Provision,
                    PartialEvidence::NothingObserved,
                    DiagnosticArtifact::of_error(&failure.error),
                )?,
                failure.release,
            )
        }
    };

    let PublishedAttempt {
        provenance,
        database_identity,
        resources,
    } = published;
    let driven = drive(resources.server(), &database_identity, attempt);
    // Client first, then server and staged module — a connection outliving the server it is
    // connected to would be reported as an abnormal disconnect this teardown order caused.
    // `drive` has already released the client by the time it returns.
    let teardown = resources.teardown();

    let (settlement, mut release_errors) = driven?;
    if let Err(error) = teardown {
        release_errors.push(error);
    }
    let release = released(release_errors);

    match settlement {
        MeasuredSettlement::NotMeasured { failure } => CalibrationRecord::not_measured(
            attempt, pinned, seed, provenance, failure, release,
        ),
        MeasuredSettlement::Unbracketed { before, failure } => CalibrationRecord::unbracketed(
            attempt, pinned, seed, provenance, before, failure, release,
        ),
        MeasuredSettlement::Bracketed { host, outcome } => {
            CalibrationRecord::attempted(attempt, pinned, seed, provenance, host, outcome, release)
        }
    }
}

/// Everything publication established, plus the capabilities the caller must release.
struct PublishedAttempt {
    provenance: AttemptProvenance,
    database_identity: String,
    resources: RunResources,
}

/// A provisioning failure with the greatest verified prefix it reached and what became of anything
/// it had acquired.
///
/// Three fields rather than one error, because the record model keeps them apart: the prefix is the
/// retained provisioning fact, the error is the diagnostic, and the release is a separate truthful
/// claim about cleanup.
struct ProvisionFailure {
    partial: PartialProvision,
    error: Error,
    release: ResourceDisposition,
}

/// Walk the monotone provisioning prefix, publishing the hash-verified module at its end.
///
/// **The accepted discovery screen's `provision` copied exactly**, because this pilot pins the same
/// artifacts and adds no module change; re-deriving the prefix, its dispositions, or the order of
/// its steps would be a second acquisition scheme to get right for no gain. Each step names the
/// prefix a failure there leaves behind, so [`PartialProvision`] is built where the fact is
/// established rather than reconstructed later from a diagnostic string. The dispositions follow the
/// source, not a guess:
///
/// - resolving the distribution only reads paths out of the Nix store, so nothing is owned yet;
/// - [`StagedModuleWasm::load`] creates its tempfile locally, so a failure inside it drops that
///   tempfile before any `StagedModuleWasm` exists;
/// - [`RunningPinnedServer::start`] reaps its own child and removes its own data directory before
///   returning `Err`, so the staged module is all this driver still holds;
/// - [`RunningPinnedServer::publish`] takes `&self`, so a publication failure leaves *both* the
///   started server and the staged module owned here.
fn provision(
    listen: ListenAddress,
    module_wasm: &Path,
) -> Result<PublishedAttempt, ProvisionFailure> {
    let distribution = match VerifiedDistribution::resolve() {
        Ok(distribution) => distribution,
        Err(error) => {
            return Err(ProvisionFailure {
                partial: PartialProvision::NothingResolved,
                error,
                release: ResourceDisposition::NotAcquired,
            })
        }
    };
    let distribution_facts = DistributionFacts::observed(&distribution);

    let staged = match StagedModuleWasm::load(module_wasm, WasmSha256::new(MODULE_WASM_SHA256)) {
        Ok(staged) => staged,
        Err(error) => {
            return Err(ProvisionFailure {
                partial: PartialProvision::DistributionResolved {
                    distribution: distribution_facts,
                },
                error,
                release: ResourceDisposition::NotAcquired,
            })
        }
    };
    let staged_wasm_sha256 = staged.sha256();

    let server = match RunningPinnedServer::start(&distribution, listen) {
        Ok(server) => server,
        Err(error) => {
            let mut errors = Vec::new();
            if let Err(cleanup) = staged.cleanup() {
                errors.push(cleanup);
            }
            return Err(ProvisionFailure {
                partial: PartialProvision::ModuleStaged {
                    distribution: distribution_facts,
                    staged_wasm_sha256,
                },
                error,
                release: released(errors),
            });
        }
    };

    let artifact = match server.publish(&distribution, &staged) {
        Ok(artifact) => artifact,
        Err(error) => {
            // Snapshotted before the bundle below moves `server`, because this is the last point at
            // which the proven process facts are readable.
            let partial = PartialProvision::ServerStarted {
                distribution: distribution_facts,
                staged_wasm_sha256,
                server: ServerFacts::observed(&server),
            };
            let mut errors = Vec::new();
            if let Err(teardown) = RunResources::new(server, staged).teardown() {
                errors.push(teardown);
            }
            return Err(ProvisionFailure {
                partial,
                error,
                release: released(errors),
            });
        }
    };

    let provenance = AttemptProvenance::observed(&distribution, &server, &artifact);
    let database_identity = artifact.database_identity().canonical_hex();
    Ok(PublishedAttempt {
        provenance,
        database_identity,
        resources: RunResources::new(server, staged),
    })
}

/// Which post-publication record shape one attempt's evidence supports, with exactly that shape's
/// facts.
///
/// The intermediate the driver needs and the record model deliberately does not provide: a
/// `CalibrationRecord` also carries identity, pinned artifacts, seed, provenance, and disposition,
/// none of which the measurement knows or should. Three variants, one per post-publication shape, so
/// the mapping at the end of [`run_attempt`] is total and holds no room for a shape whose facts are
/// missing.
enum MeasuredSettlement {
    /// The measurement window never opened, so there is no host observation.
    NotMeasured { failure: AttemptFailure },
    /// The `after` observation failed, so only the `before` one survives.
    Unbracketed {
        before: EnvironmentSample,
        failure: AttemptFailure,
    },
    /// The window opened and closed with both observations.
    Bracketed {
        host: HostObservations,
        outcome: AttemptedOutcome,
    },
}

impl MeasuredSettlement {
    /// A failure before the measurement window opened. Always carries `NothingObserved`, which is
    /// not a choice: every `Unmeasured` kind strictly precedes the first append's issue.
    fn not_measured(kind: FailureKind, error: Error) -> Result<Self> {
        Ok(Self::NotMeasured {
            failure: AttemptFailure::observed(
                kind,
                PartialEvidence::NothingObserved,
                DiagnosticArtifact::of_error(&error),
            )?,
        })
    }

    /// A failure that left the bracket open, retaining the `before` observation it did take and
    /// whatever the batch had already recorded.
    fn unbracketed(
        before: EnvironmentSample,
        kind: FailureKind,
        partial: PartialEvidence,
        error: Error,
    ) -> Result<Self> {
        Ok(Self::Unbracketed {
            before,
            failure: AttemptFailure::observed(kind, partial, DiagnosticArtifact::of_error(&error))?,
        })
    }

    /// A failure inside a bracket that did close, so the record holds both observations.
    fn bracketed_failure(
        host: HostObservations,
        kind: FailureKind,
        partial: PartialEvidence,
        error: Error,
    ) -> Result<Self> {
        Ok(Self::Bracketed {
            host,
            outcome: AttemptedOutcome::Failed {
                failure: AttemptFailure::observed(
                    kind,
                    partial,
                    DiagnosticArtifact::of_error(&error),
                )?,
            },
        })
    }
}

/// Connect the subscriber, measure, then release it — returning the settlement together with
/// whatever the release failed at.
///
/// The release errors travel back rather than being raised, because they are the caller's
/// [`ResourceDisposition`] and not this attempt's outcome: a measurement that succeeded and a client
/// that would not disconnect are two separate facts, and collapsing them would either discard the
/// evidence or claim a clean release.
fn drive(
    server: &RunningPinnedServer,
    database_identity: &str,
    attempt: AttemptKey,
) -> Result<(MeasuredSettlement, Vec<Error>)> {
    let server_url = server.listen().client_url();
    let client = match ConnectedClient::connect(&server_url, database_identity) {
        Ok(client) => client,
        // `connect` reports only after releasing whatever it built, so a failure here leaves no
        // client to disconnect and no release error to carry.
        Err(error) => {
            return Ok((
                MeasuredSettlement::not_measured(FailureKind::Connect, error)?,
                Vec::new(),
            ))
        }
    };

    let settlement = measure(&client, attempt);

    let mut release_errors = Vec::new();
    if let Err(error) = client.disconnect() {
        release_errors.push(error.context("disconnecting the calibration subscriber"));
    }
    // Unwrapped only after the disconnect, for the same reason `run_attempt` unwraps only after
    // teardown: the harness-bug path must not skip a release.
    Ok((settlement?, release_errors))
}

/// Seed both populations, bracket the paced append batch, validate both caches, and settle.
///
/// **The order is the frozen method and is not free to move.**
///
/// - The identity guard runs before the first write, so an attempt that cannot have an unrelated
///   axis is refused rather than spent measuring one it does not have.
/// - Seeding completes before the arm subscription, so the subscription's initial snapshot is the
///   deterministic baseline rather than a moving target.
/// - The arm subscription completes before the `before` observation and the first append, because
///   the estimand *is* an appended row becoming visible in a cache that already exists; a
///   subscription applied mid-batch would leave the earliest samples measuring something else.
/// - The `before` observation follows seeding, so it describes the host at measurement time rather
///   than before 1,010 reducer round trips.
/// - The `after` observation precedes the witness subscription and every cache read, so the bracket
///   contains the batch and not the validation that follows it.
fn measure(client: &ConnectedClient, attempt: AttemptKey) -> Result<MeasuredSettlement> {
    // The earliest point this can run — the measured identity is server-issued at the handshake, so
    // it is unknowable before the connection exists — and the last point before any *experimental*
    // mutation. The connection and its handshake have already happened; nothing has been written to
    // the database, subscribed, or measured. A refusal here therefore costs one connection, which
    // `drive` releases on this path exactly as it does on every other, and no seeded state at all.
    //
    // Left unchecked, the arm would hold both populations, verification would fail closed on the
    // unrelated-range rows, and the terminal record would name a sender-scope leak — the gravest
    // charge against this candidate — where the real fault is an identity collision.
    let measured = client.measured_identity();
    if let Some(settlement) = guard_unrelated_axis(measured)? {
        return Ok(settlement);
    }

    let rung = attempt.rung();
    println!(
        "  seeding {} unrelated rows and {SUBSCRIBER_OWN_ROWS} own rows",
        rung.unrelated_rows(),
    );
    if let Err(error) = seed_populations(client, rung, measured) {
        return MeasuredSettlement::not_measured(FailureKind::Reducer, error);
    }

    let target = attempt.timed_target();
    let arm = match client.subscribe_untimed(target.subscription_sql()) {
        Ok(subscription) => subscription,
        Err(error) => {
            return MeasuredSettlement::not_measured(
                FailureKind::Subscription,
                error.context(format!(
                    "subscribing the measured arm {}",
                    target.canonical_tag()
                )),
            )
        }
    };

    // Every append the batch will issue is built here, and the progress line is printed here, before
    // the bracket opens. The row recipe — key arithmetic, timestamp derivation, owner and control
    // attribution — must not run inside any measured interval, and the `println!` must not run inside
    // the bracket: neither would touch a sample, whose clock starts inside the batch loop, but the
    // bracket is a claim about the host across the measured window and widening it with harness I/O
    // would make that claim describe something the batch did not do. The accepted screen prepares
    // everything before its bracket for the same reason.
    let appends = paced_appends(measured);
    println!("  appending {MAX_PACED_SAMPLES_USIZE} paced single-row transactions");

    // The bracket's own origin, anchored immediately before the `before` reading so that reading's
    // offset is genuinely its zero and the `after` offset measures the interval between the two.
    let bracket_origin = Instant::now();
    let before = match observe_environment(BRACKET_ORIGIN_OFFSET_NANOS) {
        Ok(before) => before,
        Err(error) => {
            return MeasuredSettlement::not_measured(FailureKind::HostObservationBefore, error)
        }
    };

    let batch = client.measure_paced_appends(&appends);

    // Attempted immediately after the batch returns whether or not it completed, and before the
    // witness subscription or any cache read.
    let after = observe_after(bracket_origin);

    let (samples, host) = match (batch, after) {
        // The batch stopped early *and* the bracket could not be closed. Both diagnostics travel in
        // one chained error because either alone would misdescribe what happened, and the samples
        // the batch did complete are retained.
        (Err(batch), Err(after)) => {
            return MeasuredSettlement::unbracketed(
                before,
                FailureKind::HostObservationAfterBatchFailure,
                PartialEvidence::RejectedSeries {
                    series: RejectedSeries::of(retained(&batch.samples)),
                },
                into_error(vec![batch.failure.into_error(), after]),
            )
        }
        // The batch stopped early but the bracket closed, so the record holds both observations and
        // however many samples the batch reached — empty if it failed on its very first append.
        (Err(batch), Ok(after)) => {
            return MeasuredSettlement::bracketed_failure(
                HostObservations::bracketing(before, after),
                FailureKind::PacedBatch,
                PartialEvidence::RejectedSeries {
                    series: RejectedSeries::of(retained(&batch.samples)),
                },
                batch.failure.into_error(),
            )
        }
        // The batch ran to its end, so its samples survive the observation that failed after them.
        (Ok(samples), Err(after)) => {
            return MeasuredSettlement::unbracketed(
                before,
                FailureKind::HostObservationAfter,
                PartialEvidence::RejectedSeries {
                    series: RejectedSeries::of(retained(&samples)),
                },
                after,
            )
        }
        (Ok(samples), Ok(after)) => (samples, HostObservations::bracketing(before, after)),
    };

    // The untimed composition witness, issued only now that the bracket has closed, so it adds no
    // subscription to the connection during the measured window.
    let witness_target = CalibrationTarget::CompositionWitness;
    let witness = match client.subscribe_untimed(witness_target.subscription_sql()) {
        Ok(subscription) => subscription,
        Err(error) => {
            return MeasuredSettlement::bracketed_failure(
                host,
                FailureKind::WitnessSubscription,
                PartialEvidence::RejectedSeries {
                    series: RejectedSeries::of(retained(&samples)),
                },
                error.context(format!(
                    "subscribing the untimed composition witness {}",
                    witness_target.canonical_tag()
                )),
            )
        }
    };

    // A harness bug if it fails, and propagated as one: the guard above proved the identity is not
    // the unrelated owner, and the batch returned `Ok` only by completing every append in a list of
    // exactly `MAX_PACED_SAMPLES`, so both of this constructor's preconditions are already
    // established. There is no runtime condition left for it to reject.
    let recorded_appends = u64::try_from(samples.len())
        .expect("a completed batch is exactly the frozen sample count, which fits u64");
    let expected = CalibrationExpectation::after(rung, recorded_appends, measured).context(
        "building the expectation for a batch that completed every frozen append as the connected \
         identity",
    )?;

    // Reads reached only through the value that owns both live subscriptions.
    let composition = LiveSubscriptions::both_applied(arm, witness).verify(client, expected);

    // Composition is judged first, and the order is load-bearing: a `Sample` failure is *defined* as
    // one whose caches agreed, so a mismatch is a semantic failure whatever the samples turned out
    // to be. Sealing first would let a mismatched attempt whose series was also unsealable be filed
    // as a sample fault, hiding the mismatch.
    let population = match composition {
        Ok(population) => population,
        Err(mismatch) => {
            let error = anyhow!(
                "the row-by-row composition check failed, so this attempt is invalid: {}",
                mismatch.faults().join("; "),
            );
            return MeasuredSettlement::bracketed_failure(
                host,
                FailureKind::Semantics,
                PartialEvidence::RejectedSeriesAndMismatch {
                    series: RejectedSeries::of(retained(&samples)),
                    mismatch,
                },
                error,
            );
        }
    };

    let sealed = retained(&samples);
    match CalibrationSeries::recorded(sealed.clone(), population) {
        Ok(series) => Ok(MeasuredSettlement::Bracketed {
            host,
            outcome: AttemptedOutcome::CalibrationRecorded { series },
        }),
        // The composition matched above, so the only gates left to refuse are the series' own
        // length and positivity.
        Err(error) => MeasuredSettlement::bracketed_failure(
            host,
            FailureKind::Sample,
            PartialEvidence::RejectedSeries {
                series: RejectedSeries::of(sealed),
            },
            error,
        ),
    }
}

/// Refuse an aliased measured identity, as the terminal settlement it must become.
///
/// `None` means the precondition holds and the attempt proceeds; `Some` is this attempt's whole
/// outcome. The outer `Result` is the harness-bug path shared by every settlement constructor.
///
/// **Split out of [`measure`] because it is the one step of the measurement path that depends on
/// nothing but an identity.** Everything else there needs a live server; this needs a 32-byte value.
/// Factoring it makes the entire precondition — that the guard fires, which kind it becomes, which
/// record shape that kind admits, and what evidence such a failure may carry — provable without
/// provisioning anything, which is exactly the property a guard against a vanishingly rare
/// configuration must have if it is ever to be checked at all.
fn guard_unrelated_axis(measured: Identity) -> Result<Option<MeasuredSettlement>> {
    match ensure_measured_is_not_the_unrelated_owner(measured) {
        Ok(()) => Ok(None),
        Err(error) => {
            MeasuredSettlement::not_measured(FailureKind::IdentityCollision, error).map(Some)
        }
    }
}

/// Mint the pilot's retained sample representation from the client's raw measured intervals.
///
/// The conversion exists because the two types face opposite ways. [`LatencySample`] is the shared
/// measurement vocabulary and hands its nanoseconds back; [`PacedSampleNanos`] is this module's
/// visibility boundary and does not. Crossing it here, once, is what keeps every retained sample —
/// admitted or rejected — on the far side of that boundary.
fn retained(samples: &[LatencySample]) -> Vec<PacedSampleNanos> {
    samples
        .iter()
        .map(|sample| PacedSampleNanos::of(sample.nanos()))
        .collect()
}

/// The two subscription handles this call site supplies, retained across both cache reads.
///
/// **What this type actually guarantees is narrow: the two handles handed to
/// [`Self::both_applied`] are still owned when [`Self::verify`] performs its reads.** Because
/// `verify` borrows `self`, neither handle can be dropped between the two reads without failing to
/// compile. The composition is one claim about one instant, and a cache read after its subscription
/// had ended would report an emptiness the seeding did not cause and be recorded as a `Semantics`
/// mismatch — a wrong verdict, not a detected one. Keeping the handles alive across the reads is the
/// part of that hazard this type removes.
///
/// **What it does not establish**, and must not be read as establishing:
///
/// - that the two handles are *distinct*, or that they are the arm and witness subscriptions rather
///   than the same one passed twice — nothing in the constructor's types says so, and the field
///   names are the only record of the intent;
/// - that no *other* holder unsubscribes either query while `verify` runs. `unsubscribe` /
///   `unsubscribe_then` consume `self`, so this owner cannot, but ownership here says nothing about
///   the wider connection.
///
/// Under pinned SDK 2.7.0 dropping a handle would not end its subscription anyway —
/// `SubscriptionHandleImpl` has no `Drop` impl — so the retention this type enforces is belt and
/// braces today. It is still worth a type, because that property rests on the *absence* of an impl
/// in an undocumented internal, which nothing in this repository would notice changing.
struct LiveSubscriptions {
    /// Retained but never inspected: what matters is that it is still alive, not what it holds. The
    /// arm's subscription is also the one the whole paced batch measured into, applied long before
    /// this value exists.
    _arm: SubscriptionHandle,
    _witness: SubscriptionHandle,
}

impl LiveSubscriptions {
    /// Both subscriptions, applied and live.
    fn both_applied(arm: SubscriptionHandle, witness: SubscriptionHandle) -> Self {
        Self {
            _arm: arm,
            _witness: witness,
        }
    }

    /// Read both caches and validate every row against the frozen expectation, with both
    /// subscriptions still live.
    ///
    /// Both, not just the arm: an attempt whose arm held exactly its own slice has shown nothing
    /// about whether the unrelated population it was supposed to sit on top of was actually seeded,
    /// and a series recorded against an empty backing table is not a series recorded at the baseline
    /// rung.
    ///
    /// Takes `&self` purely for that guarantee — it reads nothing out of the handles, and the borrow
    /// is the point.
    fn verify(
        &self,
        client: &ConnectedClient,
        expected: CalibrationExpectation,
    ) -> Result<VerifiedPopulation, CompositionMismatch> {
        let arm: Vec<ObservedRow> = client
            .read_indexed_control_activity_sender_view()
            .iter()
            .map(observed)
            .collect();
        let witness: Vec<ObservedRow> = client
            .read_indexed_control_activity()
            .iter()
            .map(observed)
            .collect();
        VerifiedPopulation::verify(expected, &arm, &witness)
    }
}

/// Flatten one generated row into the verifier's harness-side snapshot.
///
/// The timestamp is converted to raw microseconds here, at the boundary, because the frozen recipe
/// *derives* it arithmetically from the id and the verifier checks exact equality against that
/// derivation — see [`expected_ts_micros`]. Doing it at the read keeps the verifier free of the
/// exact-version-coupled generated type, which is why [`ObservedRow`] exists separately at all.
fn observed(row: &IndexedControlActivity) -> ObservedRow {
    ObservedRow {
        id: row.id,
        ts_micros: row.ts.to_micros_since_unix_epoch(),
        control_uuid: row.control_uuid,
        user_identity: row.user_identity,
    }
}

/// Seed this attempt's two populations: the unrelated backing rows, then the subscriber's own slice.
///
/// **One row per transaction, and no bulk path exists to take instead.** SSOT §562 permits no
/// multi-row transaction claim for this experiment, and the module offers only the single-row
/// `insert_indexed_control_activity`; adding a bulk seeder would change the pinned WASM, and with it
/// the accepted generated-tree digest this pilot is frozen against. So both populations are
/// established the same way the measured appends are, one confirmed round trip at a time.
///
/// Both populations are written through the *same* reducer, which is why it takes an explicit
/// `user_identity` rather than deriving one from the sender: the unrelated rows must be owned by an
/// identity this connection is not, or the sender-scoped arm would return them and there would be no
/// unrelated axis. That the two identities differ is proven before this is called.
///
/// The unrelated rows are seeded first only because the ordering is arbitrary and one had to be
/// chosen: every row's timestamp is derived from its own id, so the populations are order-independent
/// and no interleaving could change what either ends up holding.
fn seed_populations(
    client: &ConnectedClient,
    rung: CalibrationRung,
    measured: Identity,
) -> Result<()> {
    let unrelated_rows = rung.unrelated_rows();
    for offset in 0..unrelated_rows {
        let id = unrelated_activity_id(offset);
        client
            .insert_indexed_control_activity(
                id,
                activity_ts(id),
                UNRELATED_CONTROL_UUID,
                unrelated_owner(),
            )
            .with_context(|| format!("seeding unrelated row {offset} of {unrelated_rows}"))?;
    }

    for offset in 0..SUBSCRIBER_OWN_ROWS {
        let id = own_activity_id(offset);
        client
            .insert_indexed_control_activity(id, activity_ts(id), OWN_CONTROL_UUID, measured)
            .with_context(|| format!("seeding own row {offset} of {SUBSCRIBER_OWN_ROWS}"))?;
    }

    Ok(())
}

/// Every append the measured batch will issue, in order.
///
/// The measured mutation is a production-shaped single-row append to the subscriber's *own* slice,
/// so each row carries the measured identity and the own control, and takes the next id above the
/// seeded slice. Built in one pass before the bracket opens so no arithmetic falls inside a measured
/// interval.
///
/// **The `S₀ + i` depth this implies is a success-path statement only.** On a batch that completes
/// every append, the own slice is `S₀ + i` rows deep before sample `i` and `S₀ + recorded` deep at
/// the end, which is exactly the expectation [`CalibrationExpectation::after`] is built from. On a
/// failure path it need not hold: an append that timed out at the barrier, or whose visibility never
/// arrived, may still have committed server-side, leaving the table one row deeper than the retained
/// sample count. That discrepancy is unobservable to this module because the expectation is
/// constructed only after the batch returns `Ok` having completed every frozen append — a failed or
/// timed-out batch never reaches it, and its retained samples are recorded as a
/// [`RejectedSeries`] that asserts no cardinality at all.
fn paced_appends(measured: Identity) -> Vec<PacedAppend> {
    (0..u64::from(MAX_PACED_SAMPLES))
        .map(|sample| {
            let offset = SUBSCRIBER_OWN_ROWS.checked_add(sample).expect(
                "the compile-time freeze proves the seeded slice plus every append fits u64",
            );
            let id = own_activity_id(offset);
            PacedAppend {
                id,
                ts: activity_ts(id),
                control_uuid: OWN_CONTROL_UUID,
                user_identity: measured,
            }
        })
        .collect()
}

/// The `offset`-th activity id in the measured identity's own key range.
fn own_activity_id(offset: u64) -> u64 {
    OWN_ACTIVITY_ID_BASE
        .checked_add(offset)
        .expect("the compile-time freeze proves the own key range fits u64")
}

/// The `offset`-th activity id in the unrelated population's key range.
fn unrelated_activity_id(offset: u64) -> u64 {
    UNRELATED_ACTIVITY_ID_BASE
        .checked_add(offset)
        .expect("the compile-time freeze proves the unrelated key range ends below the own range")
}

/// The timestamp the frozen recipe assigns to activity `id`.
///
/// Derived through [`expected_ts_micros`], the same function the verifier checks against, so a
/// written row and its expectation cannot come from two copies of one recipe.
fn activity_ts(id: u64) -> Timestamp {
    Timestamp::from_micros_since_unix_epoch(expected_ts_micros(id))
}

/// The `before` observation's offset. Zero because it *is* the bracket's origin — the `after`
/// reading's offset is then the elapsed interval between the two, which is the separation the pair
/// claims.
const BRACKET_ORIGIN_OFFSET_NANOS: u64 = 0;

/// Read the host immediately after the paced batch returns, offset from the bracket's own origin.
fn observe_after(bracket_origin: Instant) -> Result<EnvironmentSample> {
    let elapsed = bracket_origin.elapsed();
    let offset_nanos = u64::try_from(elapsed.as_nanos()).with_context(|| {
        format!("{elapsed:?} since the bracket origin does not fit u64 nanoseconds")
    })?;
    observe_environment(offset_nanos)
}

/// Collapse a release's errors into the one truthful disposition they support.
///
/// Empty means every resource this driver owned came back; anything else is a release that was
/// attempted and failed, whose diagnostic names what may remain outstanding. Never `NotAcquired` —
/// that is a claim about what was *owned*, which only a caller knows.
fn released(errors: Vec<Error>) -> ResourceDisposition {
    match errors.is_empty() {
        true => ResourceDisposition::Released,
        false => ResourceDisposition::ReleaseFailed {
            diagnostic: DiagnosticArtifact::of_error(&into_error(errors)),
        },
    }
}

/// Settle every slot in `remaining` as `NotRun` under one reason.
///
/// Separated from the append so the settlement is a pure, testable function: the frozen inventory
/// promises one terminal record per predeclared attempt, and a run that stops early keeps that
/// promise only if the slots it never reached are recorded rather than omitted.
///
/// **Infallible, so building the suffix's records cannot fail.** [`CalibrationRecord::not_run`] has
/// nothing to reject, so neither has this: one input slot yields exactly one output record, always.
///
/// That is a local guarantee and no more. Once this function is invoked, record *construction*
/// cannot fail; the append and flush that follow still can, and this says nothing about whether the
/// function is reached — a harness bug propagating out of [`run_attempt`] abandons the suffix before
/// settlement is invoked at all. Both of those are the module header's two stated exceptions to
/// per-attempt terminal coverage, and neither is closed here.
fn settle_remaining(
    remaining: &[AttemptKey],
    pinned: &PinnedArtifactIdentity,
    seed: u64,
    reason: &NotRunReason,
) -> Vec<CalibrationRecord> {
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
    append_all(ledger, &settle_remaining(remaining, pinned, seed, &reason))?;
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

#[cfg(test)]
mod tests;
