//! Driver for the `ControlRegistry` discovery E3 screen.
//!
//! The frozen inventory, its execution order, gate totality, per-attempt acquisition, and the
//! durable ledger seam all live here. Each attempt provisions its own pinned instance, publishes the
//! hash-verified module, seeds its rung's history through the two atomic registry reducers, brackets
//! one callback-first cold subscription apply with host observations, validates all four caches, and
//! tears down.
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
use std::time::Instant;

use anyhow::{anyhow, ensure, Context, Error, Result};
use spacetimedb_sdk::{Identity, Timestamp};

use crate::client::connected_client::ConnectedClient;
use crate::control_registry_discovery_screen::attempt_failure::AttemptFailure;
use crate::control_registry_discovery_screen::attempt_inventory::AttemptInventory;
use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::attempted_outcome::AttemptedOutcome;
use crate::control_registry_discovery_screen::cold_apply_evidence::ColdApplyEvidence;
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::four_way_composition::FourWayComposition;
use crate::control_registry_discovery_screen::four_way_expectation::FourWayExpectation;
use crate::control_registry_discovery_screen::four_way_observation::FourWayObservation;
use crate::control_registry_discovery_screen::host_observations::HostObservations;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::partial_evidence::PartialEvidence;
use crate::control_registry_discovery_screen::partial_provision::PartialProvision;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::rejected_apply_nanos::RejectedApplyNanos;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::retry_ordinal::RetryOrdinal;
use crate::control_registry_discovery_screen::screen_params::{
    CONTROL_UUID_BASE, IDENTITY_BYTE_BASE, REGISTRY_CONTROLS, TS_BASE_MICROS, TS_STEP_MICROS,
};
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::control_registry_discovery_screen::screen_target::ScreenTarget;
use crate::entity_owner_pilot::attempt_provenance::{
    AttemptProvenance, DistributionFacts, ServerFacts,
};
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::bindings::SubscriptionHandle;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::observation::output_path::OutputPath;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;
use crate::view_read_set_campaign::campaign_driver::observe_environment;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

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
/// Provisions a verified pinned distribution and fresh server/data directory, publishes the
/// hash-verified module, seeds `K` controls with `N / K` history rows each through the two atomic
/// registry reducers only, takes the `before` host observation immediately before issuing the timed
/// subscription, stops the clock inside `on_applied` as its first statement, takes the `after`
/// observation immediately after the timed interval and before any validation subscription or cache
/// read, then subscribes the other three targets, validates all four caches, settles, and tears
/// down.
///
/// **Returns a record rather than an error for every attempt-level failure.** Each of the ten
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
    seed: ScheduleSeed,
) -> Result<ScreenRecord> {
    let published = match provision(listen, module_wasm) {
        Ok(published) => published,
        Err(failure) => {
            return ScreenRecord::not_provisioned(
                attempt,
                pinned,
                seed,
                ORIGINAL_HAS_NO_SUPERSESSION,
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
        MeasuredSettlement::NotMeasured { failure } => ScreenRecord::not_measured(
            attempt,
            pinned,
            seed,
            ORIGINAL_HAS_NO_SUPERSESSION,
            provenance,
            failure,
            release,
        ),
        MeasuredSettlement::Unbracketed { before, failure } => ScreenRecord::unbracketed(
            attempt,
            pinned,
            seed,
            ORIGINAL_HAS_NO_SUPERSESSION,
            provenance,
            before,
            failure,
            release,
        ),
        MeasuredSettlement::Bracketed { host, outcome } => ScreenRecord::attempted(
            attempt,
            pinned,
            seed,
            ORIGINAL_HAS_NO_SUPERSESSION,
            provenance,
            host,
            outcome,
            release,
        ),
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
/// claim about cleanup. Folding a cleanup failure into the primary error — which the smoke and probe
/// drivers do, having no disposition field to put it in — would make a record unable to say whether
/// a resource is still outstanding.
struct ProvisionFailure {
    partial: PartialProvision,
    error: Error,
    release: ResourceDisposition,
}

/// Walk the monotone provisioning prefix, publishing the hash-verified module at its end.
///
/// Each step names the prefix a failure there leaves behind, so [`PartialProvision`] is built where
/// the fact is established rather than reconstructed later from a diagnostic string. The dispositions
/// follow the source, not a guess:
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
/// `ScreenRecord` also carries identity, pinned artifacts, seed, provenance, and disposition, none
/// of which the measurement knows or should. Three variants, one per post-publication shape, so the
/// mapping at the end of [`run_attempt`] is total and holds no room for a shape whose facts are
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
    /// not a choice: every `Unmeasured` kind strictly precedes the timed subscription's issue.
    fn not_measured(kind: FailureKind, error: Error) -> Result<Self> {
        Ok(Self::NotMeasured {
            failure: AttemptFailure::observed(
                kind,
                PartialEvidence::NothingObserved,
                DiagnosticArtifact::of_error(&error),
            )?,
        })
    }

    /// A failure that left the bracket open, retaining the `before` observation it did take.
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
        release_errors.push(error.context("disconnecting the screen subscriber"));
    }
    // Unwrapped only after the disconnect, for the same reason `run_attempt` unwraps only after
    // teardown: the harness-bug path must not skip a release.
    Ok((settlement?, release_errors))
}

/// Seed this rung's history, bracket one cold apply, validate all four caches, and settle.
///
/// The order is the frozen method and is not free to move: seeding completes before the `before`
/// observation, so the observation describes the host at measurement time rather than before up to
/// 32,000 reducer round trips; the `after` observation precedes every validation subscription and
/// cache read, so the bracket measures the timed interval and not the validation that follows it.
fn measure(client: &ConnectedClient, attempt: AttemptKey) -> Result<MeasuredSettlement> {
    let rung = attempt.rung();
    println!(
        "  seeding N={} history rows across K={REGISTRY_CONTROLS} controls ({} each)",
        rung.history_rows(),
        rung.rows_per_control(),
    );
    if let Err(error) = seed_history(client, rung) {
        return MeasuredSettlement::not_measured(FailureKind::Reducer, error);
    }

    // Everything the timed subscription needs is prepared *before* the bracket opens, so that the
    // `before` reading really is immediately before the subscription is issued. Preparing them
    // afterwards would put two `format!`s between the observation and the measurement and make
    // "immediately before" false; it would also put the query inside the sample, which the frozen
    // estimand — cold subscription materialization plus cache apply — excludes.
    let target = attempt.target();
    let sql = target.subscription_sql();
    let description = target.canonical_tag().to_string();

    // The bracket's own origin, anchored immediately before the `before` reading so that reading's
    // offset is genuinely its zero and the `after` offset measures the interval between the two.
    // The campaign's gate origin is not reused: it belongs to a prospective decision this bracket
    // has nothing to do with.
    let bracket_origin = Instant::now();
    let before = match observe_environment(BRACKET_ORIGIN_OFFSET_NANOS) {
        Ok(before) => before,
        Err(error) => {
            return MeasuredSettlement::not_measured(FailureKind::HostObservationBefore, error)
        }
    };

    // Nothing between the observation above and the SDK issue but the helper's own disclosed fixed
    // residue: one `mpsc` channel and two boxed callbacks.
    let timed = client.subscribe_cold_target(sql, description);

    // Attempted immediately after the timed interval whether or not it applied, and before any
    // validation subscription or cache read.
    let after = observe_after(bracket_origin);

    let (_retained_timed, apply_nanos, host) = match (timed, after) {
        // Neither a sample nor a closed bracket. Both diagnostics travel in one chained error
        // because either alone would misdescribe what happened.
        (Err(timed), Err(after)) => {
            return MeasuredSettlement::unbracketed(
                before,
                FailureKind::HostObservationAfterTimedFailure,
                PartialEvidence::NothingObserved,
                into_error(vec![timed.into_error(), after]),
            )
        }
        // The apply failed but the bracket closed, so the record holds both observations and no
        // sample — the one case in which a `Bracketed` failure has nothing to retain.
        (Err(timed), Ok(after)) => {
            return MeasuredSettlement::bracketed_failure(
                HostObservations::bracketing(before, after),
                FailureKind::TimedSubscription,
                PartialEvidence::NothingObserved,
                timed.into_error(),
            )
        }
        // The apply completed, so its raw duration survives the observation that failed after it.
        (Ok((_subscription, applied)), Err(after)) => {
            return MeasuredSettlement::unbracketed(
                before,
                FailureKind::HostObservationAfter,
                PartialEvidence::RejectedSample {
                    apply_nanos: RejectedApplyNanos::of(applied.nanos()),
                },
                after,
            )
        }
        // The handle is moved into `LiveSubscriptions` below rather than dropped: the reads are
        // reached only through the value that owns it.
        (Ok((subscription, applied)), Ok(after)) => (
            subscription,
            applied.nanos(),
            HostObservations::bracketing(before, after),
        ),
    };

    // The three untimed validation subscriptions, issued only now that the bracket has closed. Any
    // one of them failing returns here, so the array below is reached only with all three applied.
    let mut validation = Vec::with_capacity(VALIDATION_TARGET_COUNT);
    for other in validation_targets(target) {
        match client.subscribe_untimed(other.subscription_sql()) {
            Ok(subscription) => validation.push(subscription),
            Err(error) => {
                return MeasuredSettlement::bracketed_failure(
                    host,
                    FailureKind::ValidationSubscription,
                    PartialEvidence::RejectedSample {
                        apply_nanos: RejectedApplyNanos::of(apply_nanos),
                    },
                    error.context(format!(
                        "subscribing the untimed validation target {}",
                        other.canonical_tag()
                    )),
                )
            }
        }
    }
    let validation = validation.try_into().unwrap_or_else(|applied: Vec<_>| {
        panic!(
            "the loop pushes one handle per validation target, so {VALIDATION_TARGET_COUNT} must \
             be live here, not {}",
            applied.len(),
        )
    });

    // Reads reached only through the value that owns all four live subscriptions.
    let composition =
        LiveSubscriptions::all_applied(_retained_timed, validation).read_composition(client, rung);

    // Composition is judged first, and the order is load-bearing: a `Sample` failure is *defined* as
    // one whose four caches agreed, so a mismatch is a semantic failure whatever the duration turned
    // out to be. Sealing first would let a mismatched attempt whose duration was also nonpositive be
    // filed as a nonpositive statistic, hiding the mismatch.
    if !composition.matches() {
        return MeasuredSettlement::bracketed_failure(
            host,
            FailureKind::Semantics,
            PartialEvidence::RejectedSampleAndComposition {
                apply_nanos: RejectedApplyNanos::of(apply_nanos),
                composition,
            },
            anyhow!(
                "the four-way composition check failed, so this pair is invalid: {}",
                composition.mismatches().join("; "),
            ),
        );
    }

    match ColdApplyEvidence::sealed(apply_nanos, composition) {
        Ok(evidence) => Ok(MeasuredSettlement::Bracketed {
            host,
            outcome: AttemptedOutcome::Complete { evidence },
        }),
        // The composition matched above, so the only gate left to refuse is the duration itself.
        Err(error) => MeasuredSettlement::bracketed_failure(
            host,
            FailureKind::Sample,
            PartialEvidence::RejectedSampleAndComposition {
                apply_nanos: RejectedApplyNanos::of(apply_nanos),
                composition,
            },
            error,
        ),
    }
}

/// How many targets an attempt subscribes untimed: every one it did not time.
///
/// Derived from [`ScreenTarget::ALL`] rather than written as three, so adding a fifth target could
/// not leave a stale literal behind that silently stopped validating one cache.
const VALIDATION_TARGET_COUNT: usize = ScreenTarget::ALL.len() - 1;

/// The targets an attempt validates untimed, given the one it timed.
///
/// Exactly the complement of the timed target, so all four caches end up subscribed and none twice.
/// Returned as a fixed-size array, which is what lets the cardinality invariant travel into
/// [`LiveSubscriptions`] instead of being re-checked there.
fn validation_targets(timed: ScreenTarget) -> [ScreenTarget; VALIDATION_TARGET_COUNT] {
    let others: Vec<ScreenTarget> = ScreenTarget::ALL
        .into_iter()
        .filter(|target| *target != timed)
        .collect();
    others
        .try_into()
        .unwrap_or_else(|_| panic!("{timed:?} is one of the four targets, so three must remain"))
}

/// The subscriptions that must all be live at the instant the four caches are read.
///
/// **This type exists to make simultaneity a compile-time property rather than a comment.** The
/// four-way composition is one claim about one instant: a cache read after its subscription had
/// ended would report an emptiness the seeding did not cause, and would be recorded as a `Semantics`
/// mismatch — a wrong verdict, not a detected one. Holding the handles in a value whose method
/// performs the reads means a read cannot be moved past a drop without failing to compile, because
/// [`Self::read_composition`] borrows `self`.
///
/// Pinned SDK 2.7.0 would not in fact end a subscription on drop — `SubscriptionHandleImpl` has no
/// `Drop` impl (`subscription.rs:455`) and both `unsubscribe`/`unsubscribe_then` consume `self` — so
/// today this is belt and braces. That is exactly why it is worth a type: the property the screen
/// depends on would otherwise rest on the *absence* of an impl in an undocumented internal, which
/// nothing in this repository would notice changing.
/// **Cardinality is structural, not checked.** There is no partial state and no read method on one:
/// the only constructor takes the timed handle plus a fixed `[SubscriptionHandle;
/// VALIDATION_TARGET_COUNT]`, so a value of this type *is* four applied subscriptions. A caller that
/// subscribed two of three, or looped zero times, cannot build one, and therefore cannot reach
/// [`Self::read_composition`] to report the empty caches it would have found.
struct LiveSubscriptions {
    /// Retained but never inspected: what matters is that it is still alive, not what it holds.
    _timed: SubscriptionHandle,
    _validation: [SubscriptionHandle; VALIDATION_TARGET_COUNT],
}

impl LiveSubscriptions {
    /// All four subscriptions, applied and live.
    ///
    /// The array parameter is the invariant: the caller reaches it only by having applied one
    /// subscription per [`validation_targets`] entry, and any failure among them returns before this
    /// is called.
    fn all_applied(
        timed: SubscriptionHandle,
        validation: [SubscriptionHandle; VALIDATION_TARGET_COUNT],
    ) -> Self {
        Self {
            _timed: timed,
            _validation: validation,
        }
    }

    /// Read all four caches against the rung's frozen expectation, with every subscription still
    /// live.
    ///
    /// All four, not just the timed one: an attempt that timed Arm A and saw K rows has shown
    /// nothing about whether the history it was supposed to sit on top of was actually seeded.
    ///
    /// Takes `&self` purely for that guarantee — it reads nothing out of the handles, and the
    /// borrow is the point.
    fn read_composition(&self, client: &ConnectedClient, rung: ScreenRung) -> FourWayComposition {
        FourWayComposition::checked(
            FourWayExpectation::at(rung),
            FourWayObservation::read(
                cache_rows(client.read_control_registry_all_view().len()),
                cache_rows(client.read_control_registry().len()),
                cache_rows(client.read_control_activity_latest_by_control_view().len()),
                cache_rows(client.read_control_activity().len()),
            ),
        )
    }
}

/// The `before` observation's offset. Zero because it *is* the bracket's origin — the `after`
/// reading's offset is then the elapsed interval between the two, which is the separation the pair
/// claims.
const BRACKET_ORIGIN_OFFSET_NANOS: u64 = 0;

/// Read the host immediately after the timed interval, offset from the bracket's own origin.
fn observe_after(bracket_origin: Instant) -> Result<EnvironmentSample> {
    let elapsed = bracket_origin.elapsed();
    let offset_nanos = u64::try_from(elapsed.as_nanos()).with_context(|| {
        format!("{elapsed:?} since the bracket origin does not fit u64 nanoseconds")
    })?;
    observe_environment(offset_nanos)
}

/// One cache's cardinality as the composition vocabulary counts it.
fn cache_rows(rows: usize) -> u64 {
    u64::try_from(rows).expect("a subscribed cache holds fewer rows than u64 can count")
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

/// Seed this rung's frozen composition: one first-activity call per control, then
/// `rows_per_control - 1` repeat calls each.
///
/// **The accepted Step 1 recipe, copied exactly** — same id round-robin, same timestamp base and
/// step, same per-control identity. Step 1 proved that recipe publishes, materializes, and
/// reconverges on exactly K current rows over N history rows; re-deriving it here would be a second
/// deterministic seeding scheme to get right for no gain.
///
/// The order is forced by the reducers' own preconditions rather than chosen: a repeat call before
/// its control's first call is refused, and a second first call is refused. The history-only
/// `insert_control_activity` and the retired seed/touch pair are never called — a single use of
/// either would write an audit row without registry maintenance and falsify the composition this
/// attempt is about to validate.
fn seed_history(client: &ConnectedClient, rung: ScreenRung) -> Result<()> {
    for control_index in 0..REGISTRY_CONTROLS {
        let id = activity_id(control_index, 0);
        client
            .record_first_control_activity(
                id,
                control_uuid(control_index),
                control_identity(control_index),
                activity_ts(id),
            )
            .with_context(|| format!("first activity for control index {control_index}"))?;
    }

    for occurrence in 1..rung.rows_per_control() {
        for control_index in 0..REGISTRY_CONTROLS {
            let id = activity_id(control_index, occurrence);
            client
                .record_control_activity(id, control_uuid(control_index), activity_ts(id))
                .with_context(|| {
                    format!("activity {occurrence} for control index {control_index}")
                })?;
        }
    }

    Ok(())
}

/// The `control_uuid` of the control at `control_index`.
fn control_uuid(control_index: u64) -> u64 {
    CONTROL_UUID_BASE + control_index
}

/// A distinct fixed identity per control, so the registry's `control_uuid -> user_identity`
/// dependency is exercised by a column that actually varies across rows.
fn control_identity(control_index: u64) -> Identity {
    let mut bytes = [0u8; 32];
    bytes[31] = IDENTITY_BYTE_BASE + u8::try_from(control_index).expect("K fits u8");
    Identity::from_byte_array(bytes)
}

/// The activity id for a control's `occurrence`-th seeded row. Round-robin across controls, so ids
/// are globally unique and each control's ids increase with its occurrence.
fn activity_id(control_index: u64, occurrence: u64) -> u64 {
    occurrence * REGISTRY_CONTROLS + control_index
}

/// The timestamp for activity `id` — strictly increasing in `id` across the whole attempt, so every
/// timestamp is globally unique and each control's history is strictly increasing, which is exactly
/// what the repeat reducer requires and the latest-per-control comparator needs to be unambiguous.
fn activity_ts(id: u64) -> Timestamp {
    let offset = i64::try_from(id).expect("the activity id space fits i64");
    Timestamp::from_micros_since_unix_epoch(TS_BASE_MICROS + offset * TS_STEP_MICROS)
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
