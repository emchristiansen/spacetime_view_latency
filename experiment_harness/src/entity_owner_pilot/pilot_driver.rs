//! The Pilot-stage driver for the `EntityOwnerSenderView` candidate.

use std::path::Path;

use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use spacetimedb_sdk::Identity;

use crate::client::connected_client::ConnectedClient;
use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::attempt_outcome::AttemptOutcome;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;
use crate::entity_owner_pilot::campaign_provenance::PilotCampaignProvenance;
use crate::entity_owner_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact;
use crate::entity_owner_pilot::failure_kind::FailureKind;
use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;
use crate::entity_owner_pilot::method_validity::MethodValidity;
use crate::entity_owner_pilot::not_run_reason::NotRunReason;
use crate::entity_owner_pilot::partial_evidence::PartialEvidence;
use crate::entity_owner_pilot::pilot_params::{
    ENTITY_OWNER_PILOT_GLOBAL_SUBJECT, GLOBAL_KEY_BASE, GLOBAL_ROW_LADDER_LEN, OWNED_KEY_BASE,
    OWNED_SLICE_ROWS, PILOT_ROW_PAYLOAD,
};
use crate::entity_owner_pilot::pilot_record::PilotRecord;
use crate::entity_owner_pilot::pilot_sink::PilotSink;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::bindings::EntityOwner;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::observation::output_path::OutputPath;
use crate::params::{BATCH_SIZE, EXPERIMENT_ISSUER};
use crate::plan::run_role::RunRole;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;

/// A classified failure inside one attempt's measured window.
struct MeasuredFailure {
    kind: FailureKind,
    error: Error,
}

/// Run the whole Pilot: freeze the inventory from `seed`, create the ledger at `output`, and execute
/// every predeclared attempt in the frozen order, each on its own fresh isolated server.
///
/// The inventory is recorded before the first attempt executes, so the order the evidence is
/// interpreted against is on disk before any evidence exists — and so a reader can name every
/// predeclared slot even if the run dies before reaching it.
pub(crate) fn entity_owner_sender_view_pilot(
    listen: ListenAddress,
    module_wasm: &Path,
    output: &OutputPath,
    seed: ScheduleSeed,
) -> Result<()> {
    let inventory =
        AttemptInventory::frozen(seed).context("freezing the Pilot attempt inventory")?;

    let provenance =
        PilotCampaignProvenance::resolved().context("resolving the Pilot's campaign provenance")?;

    let mut sink = PilotSink::create(output).context("creating the Pilot ledger")?;

    // Finalize on every path once the sink exists — including a failure of the very first write, so
    // a torn opening line is still synced and accounted rather than left behind unfinalized — and
    // surface both facts rather than letting either hide the other. This is the aggregating teardown
    // discipline the provisioning capabilities use; `PilotSink` has no `Drop` backstop, so nothing
    // may `?` past this point.
    let walked = run_pilot(&mut sink, &inventory, &provenance, seed, listen, module_wasm);
    let finalized = sink.finalize().context("finalizing the Pilot ledger");

    match (walked, finalized) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(body), Ok(())) => Err(body),
        (Ok(()), Err(finalize)) => Err(finalize),
        (Err(body), Err(finalize)) => Err(anyhow!(
            "the Pilot failed and finalizing its ledger afterward also failed:\n  \
             [1] {body:#}\n  [2] {finalize:#}"
        )),
    }
}

/// Record the frozen order and campaign pins, then execute every predeclared attempt.
///
/// Split from [`entity_owner_sender_view_pilot`] so that the opening `Inventory` write is inside the
/// region its caller finalizes: a persist failure on the very first line must still reach
/// [`PilotSink::finalize`].
fn run_pilot(
    sink: &mut PilotSink,
    inventory: &AttemptInventory,
    provenance: &PilotCampaignProvenance,
    seed: ScheduleSeed,
    listen: ListenAddress,
    module_wasm: &Path,
) -> Result<()> {
    sink.write(&PilotRecord::Inventory {
        seed,
        inventory,
        provenance,
    })
    .context("recording the frozen attempt inventory, block order, and campaign provenance")?;

    run_inventory(sink, inventory, listen, module_wasm)
}

/// Execute every predeclared attempt in the frozen order, appending exactly one terminal record per
/// attempt.
///
/// An attempt's own failure is not propagated: a provision, connect, reducer, subscription, sample,
/// or semantics error terminates only that attempt while later inventory attempts continue. Two
/// conditions do stop the Pilot, and they are handled differently because the ledger's own health
/// differs:
///
/// - **A ledger persist failure** poisons the sink, which then refuses every further write — so no
///   truthful record about it could be appended. It propagates, and the durable inventory written
///   before execution is what lets a reader see which terminal slots are missing.
/// - **A release failure** leaves an acquired capability unproven-released while the ledger is still
///   healthy, so every untouched remaining slot is recorded [`AttemptOutcome::NotRun`] before
///   stopping.
fn run_inventory(
    sink: &mut PilotSink,
    inventory: &AttemptInventory,
    listen: ListenAddress,
    module_wasm: &Path,
) -> Result<()> {
    let attempts = inventory.attempts();
    for (position, attempt) in attempts.iter().enumerate() {
        // `run_attempt` writes this attempt's one terminal record itself, while its resources are
        // still owned — so the disposition is durable before anything is released, and this loop
        // never writes a second one for the same slot.
        let unreleased = run_attempt(sink, listen, module_wasm, *attempt)
            .with_context(|| format!("attempt {position} of the frozen inventory"))?;

        if let Some(diagnostic) = unreleased {
            // Only slots strictly after this one are untouched; this attempt already has its
            // terminal record, so no slot receives two.
            let not_run = AttemptOutcome::NotRun {
                reason: NotRunReason::PriorAttemptReleaseFailed { diagnostic },
            };
            let remaining = &attempts[position + 1..];
            for skipped in remaining {
                sink.write(&PilotRecord::Terminal {
                    attempt: *skipped,
                    outcome: &not_run,
                })
                .context("recording a skipped attempt after a prior attempt's release failed")?;
            }
            bail!(
                "attempt {position} could not establish that every capability it acquired was \
                 released; the remaining {} predeclared attempts were recorded NotRun rather than \
                 measured in an environment whose isolation is no longer proven",
                remaining.len(),
            );
        }
    }
    Ok(())
}

/// Execute one predeclared attempt on its own fresh isolated server, appending each rung's evidence
/// as it confirms.
///
/// Mirrors [`crate::entity_owner_smoke::entity_owner_sender_view_smoke`]'s acquisition pipeline and
/// unconditional teardown discipline exactly. It differs in only one way: a failure becomes this
/// attempt's recorded disposition instead of the caller's error, so the campaign continues. `Err` is
/// therefore reserved for a ledger persist failure alone, and `Ok` carries the release-failure
/// diagnostic when the attempt's capabilities were not provably released.
///
/// Every exit routes through [`settle`], which writes the attempt's one terminal record *before*
/// running that path's release — so at every acquisition depth the disposition is durable while the
/// capabilities that produced it are still owned, and no path can return early past a release.
fn run_attempt(
    sink: &mut PilotSink,
    listen: ListenAddress,
    module_wasm: &Path,
    attempt: AttemptKey,
) -> Result<Option<DiagnosticArtifact>> {
    // Depth 0 — nothing acquired.
    let distribution = match VerifiedDistribution::resolve() {
        Ok(distribution) => distribution,
        Err(error) => return settle(sink, attempt, Ok(provision_failure(&error)), Vec::new),
    };

    // Depth 1 — a failed load acquired nothing.
    let staged = match StagedModuleWasm::load(module_wasm, WasmSha256::new(MODULE_WASM_SHA256)) {
        Ok(staged) => staged,
        Err(error) => return settle(sink, attempt, Ok(provision_failure(&error)), Vec::new),
    };

    // Depth 2 — `staged` is live. `start` reaps its own child and data directory on failure, so
    // only the staged WASM is left to release, and it is released only after the terminal record.
    let server = match RunningPinnedServer::start(&distribution, listen) {
        Ok(server) => server,
        Err(error) => {
            return settle(sink, attempt, Ok(provision_failure(&error)), move || {
                let mut errors = Vec::new();
                if let Err(e) = staged.cleanup() {
                    errors.push(e);
                }
                errors
            })
        }
    };

    // Depth 3 — `server` and `staged` are both live. A shutdown failure here is a release failure
    // like any other: the server may still hold the listen address, so it stops the campaign
    // through `settle` rather than being folded into the publish diagnostic.
    let artifact = match server.publish(&distribution, &staged) {
        Ok(artifact) => artifact,
        Err(error) => {
            return settle(sink, attempt, Ok(provision_failure(&error)), move || {
                let mut errors = Vec::new();
                if let Err(e) = server.shutdown() {
                    errors.push(e);
                }
                if let Err(e) = staged.cleanup() {
                    errors.push(e);
                }
                errors
            })
        }
    };

    let database_identity = artifact.database_identity().canonical_hex();
    let resources = RunResources::new(server, staged);
    let server_url = resources.server().listen().client_url();

    // Depth 4 — `resources` is live, and in the connected branch the client is too. No `?` appears
    // below; every path hands its release to `settle`, which runs it after the terminal record.

    // Record which instance this attempt measured against *before* anything connects, subscribes,
    // seeds, or measures, so no evidence can exist that is not already joinable to its runtime and
    // its freshly published database. If this write fails the sink is terminal: no measurement
    // begins, and `settle` releases every capability and propagates the persist error.
    let provenance = AttemptProvenance::observed(&distribution, resources.server(), &artifact);
    if let Err(persist) = sink
        .write(&PilotRecord::Provisioned {
            attempt,
            provenance: &provenance,
        })
        .context("recording the attempt's provisioned instance")
        .map(|_seq| ())
    {
        return settle(sink, attempt, Err(persist), move || {
            let mut errors = Vec::new();
            if let Err(e) = resources.teardown() {
                errors.push(e);
            }
            errors
        });
    }

    match ConnectedClient::connect(&server_url, &database_identity) {
        Err(error) => {
            let outcome = failed(
                FailureKind::Connect,
                Vec::new(),
                &error.context("connecting the measured subscriber"),
            );
            settle(sink, attempt, Ok(outcome), move || {
                let mut errors = Vec::new();
                if let Err(e) = resources.teardown() {
                    errors.push(e);
                }
                errors
            })
        }
        Ok(client) => {
            let walked = walk_ladder(sink, &client, attempt);
            settle(sink, attempt, walked, move || {
                let mut errors = Vec::new();
                if let Err(e) = client.disconnect() {
                    errors.push(e.context("disconnecting the measured subscriber"));
                }
                if let Err(e) = resources.teardown() {
                    errors.push(e);
                }
                errors
            })
        }
    }
}

/// Close out one attempt while its capabilities are still owned: write its terminal record, then
/// release, then report.
///
/// The executable order is the invariant:
///
/// 1. **Terminal first**, so the disposition is durable before any capability is handed back —
///    skipped only when `outcome` is already a ledger persist failure, since the sink is terminal
///    and a further write would be dishonest rather than merely futile.
/// 2. **`release` always runs**, whatever step 1 did. It is a closure rather than a pre-computed
///    error list precisely so it cannot be evaluated before step 1.
/// 3. **Ledger error leads.** It terminates the campaign, with any release failure aggregated after
///    it so nothing is dropped. When the ledger is healthy, release failures become the returned
///    diagnostic instead, which the caller records against every remaining slot.
fn settle(
    sink: &mut PilotSink,
    attempt: AttemptKey,
    outcome: Result<AttemptOutcome>,
    release: impl FnOnce() -> Vec<Error>,
) -> Result<Option<DiagnosticArtifact>> {
    let ledger = match outcome {
        Ok(outcome) => write_terminal(sink, attempt, &outcome),
        Err(persist) => Err(persist),
    };

    let mut release_errors = release();

    match ledger {
        Ok(()) => Ok(release_failure(release_errors)),
        Err(persist) => {
            let mut errors = vec![persist];
            errors.append(&mut release_errors);
            Err(into_error(errors))
        }
    }
}

/// Append one attempt's single terminal record. The assigned sequence is not read back: the ledger
/// is append-only and nothing in this driver addresses a record by position.
fn write_terminal(
    sink: &mut PilotSink,
    attempt: AttemptKey,
    outcome: &AttemptOutcome,
) -> Result<()> {
    sink.write(&PilotRecord::Terminal { attempt, outcome })
        .context("recording the attempt's terminal outcome")
        .map(|_seq| ())
}

/// Seed the measured identity's fixed owned slice, subscribe this attempt's role to its target, then
/// walk every rung of the frozen ladder, appending each rung's evidence as soon as it confirms.
///
/// The owned slice is seeded *before* the subscription, so it arrives in the initial snapshot and
/// every global row afterward arrives as an incremental update — the read-set behavior under study.
/// `Err` is a ledger persist failure alone; a measured-window failure becomes a returned
/// [`AttemptOutcome::Failed`] carrying whatever rungs had already completed.
fn walk_ladder(
    sink: &mut PilotSink,
    client: &ConnectedClient,
    attempt: AttemptKey,
) -> Result<AttemptOutcome> {
    let role = attempt.role();
    let owner = client.measured_identity();
    let global = Identity::from_claims(EXPERIMENT_ISSUER, ENTITY_OWNER_PILOT_GLOBAL_SUBJECT);

    if let Err(failure) = seed_owned_slice(client, owner) {
        return Ok(failed(failure.kind, Vec::new(), &failure.error));
    }

    if let Err(error) = subscribe(client, role) {
        return Ok(failed(FailureKind::Subscription, Vec::new(), &error));
    }

    let mut rungs = Vec::with_capacity(GLOBAL_ROW_LADDER_LEN);
    for rung in GlobalRowRung::ALL {
        match measure_rung(client, role, owner, global, rung) {
            Err(failure) => return Ok(failed(failure.kind, rungs, &failure.error)),
            Ok(evidence) => {
                sink.write(&PilotRecord::Rung {
                    attempt,
                    evidence: &evidence,
                })
                .with_context(|| format!("recording rung {}'s evidence", rung.get()))?;
                rungs.push(evidence);
            }
        }
    }

    let artifact = EvidenceArtifact::sealed(rungs)
        .expect("the ladder walk pushes every rung exactly once in ascending order");
    Ok(AttemptOutcome::Complete {
        artifact,
        validity: MethodValidity::Valid,
    })
}

/// Seed the measured identity's own slice, held at [`OWNED_SLICE_ROWS`] for the whole ladder so the
/// Arm's result set stays constant while the backing table grows.
fn seed_owned_slice(
    client: &ConnectedClient,
    owner: Identity,
) -> std::result::Result<(), MeasuredFailure> {
    for offset in 0..OWNED_SLICE_ROWS {
        client
            .insert_entity_owner(OWNED_KEY_BASE + offset, owner, PILOT_ROW_PAYLOAD.to_string())
            .map_err(|error| MeasuredFailure {
                kind: FailureKind::Reducer,
                error: error.context("seeding the measured identity's owned slice"),
            })?;
    }
    Ok(())
}

/// Subscribe this attempt's role to its target: the Arm to the sender-scoped view, the Control to
/// the `entity_owner` base table it is matched against.
fn subscribe(client: &ConnectedClient, role: RunRole) -> Result<()> {
    match role {
        RunRole::Arm => client
            .subscribe_entity_owner_sender_view()
            .context("subscribing the Arm to entity_owner_sender_view"),
        RunRole::Control => client
            .subscribe_entity_owner()
            .context("subscribing the Control to the entity_owner base table"),
    }
}

/// Read this attempt's role's currently-subscribed rows out of the live client cache, issuing no new
/// subscription. Same per-role dispatch as [`subscribe`], so the read target can never drift from
/// the subscribed one.
fn read_subscribed(client: &ConnectedClient, role: RunRole) -> Vec<EntityOwner> {
    match role {
        RunRole::Arm => client.read_entity_owner_sender_view(),
        RunRole::Control => client.read_entity_owner(),
    }
}

/// Walk one rung: seed its unmeasured global prefix, measure the final [`BATCH_SIZE`] writes of its
/// increment, then verify the observed result set.
///
/// The ladder is cumulative on a single server, so a rung writes only its increment; a compile-time
/// proof in [`crate::entity_owner_pilot::pilot_params`] guarantees that increment is at least one
/// batch, so the measured writes always belong to this rung. The global keys are allocated from a
/// contiguous cursor at [`GLOBAL_KEY_BASE`], disjoint from the owned slice's, so no rung can collide
/// with another's rows or with the measured identity's.
///
/// The read-back happens strictly **after** the measured batch's confirmation barrier returns, so
/// every write of this rung is confirmed before the result set is judged — never mid-batch.
fn measure_rung(
    client: &ConnectedClient,
    role: RunRole,
    owner: Identity,
    global: Identity,
    rung: GlobalRowRung,
) -> std::result::Result<RungEvidence, MeasuredFailure> {
    let increment = rung.global_rows_increment();
    let already_seeded = rung.global_rows() - increment;
    let unmeasured = increment - BATCH_SIZE;
    let prefix_base = GLOBAL_KEY_BASE + already_seeded;

    for offset in 0..unmeasured {
        client
            .insert_entity_owner(
                prefix_base + offset,
                global,
                PILOT_ROW_PAYLOAD.to_string(),
            )
            .map_err(|error| MeasuredFailure {
                kind: FailureKind::Reducer,
                error: error.context(format!(
                    "seeding rung {}'s unmeasured global prefix",
                    rung.get()
                )),
            })?;
    }

    let latencies = client
        .measure_entity_owner_batch(prefix_base + unmeasured, global)
        .map_err(|error| MeasuredFailure {
            kind: FailureKind::Sample,
            error: error.context(format!("measuring rung {}'s confirmed batch", rung.get())),
        })?;

    let rows = read_subscribed(client, role);
    verify_result_set(role, owner, global, &rows, rung).map_err(|error| MeasuredFailure {
        kind: FailureKind::Semantics,
        error: error.context(format!("verifying rung {}'s result set", rung.get())),
    })?;

    let subscribed_rows = u64::try_from(rows.len()).expect("a subscribed row count fits u64");
    Ok(RungEvidence::observed(rung, subscribed_rows, latencies))
}

/// Verify the observed result set against what this role must return at `rung`.
///
/// For the Arm this is the candidate's security gate — the sender-scoped view returning exactly, and
/// only, the measured identity's own rows — restated at every rung, so a leak that appears only
/// under load is caught rather than assumed absent from the Smoke stage. For the Control it is the
/// matched composition check: the direct public table returns the whole seeded set, which is what
/// makes it the comparison baseline rather than an independent measurement.
///
/// Both roles are checked by **composition, not cardinality**: every row must fall in one of the two
/// preregistered key ranges, carry that range's expected owner, and carry the single fixed payload.
/// Because `entity_uuid` is the table's primary key, distinct rows have distinct keys — so range
/// membership plus a per-range count pins the exact expected key set without materializing it. A
/// count-only check would pass a Control that returned the right number of *wrong* rows.
fn verify_result_set(
    role: RunRole,
    owner: Identity,
    global: Identity,
    rows: &[EntityOwner],
    rung: GlobalRowRung,
) -> Result<()> {
    let global_rows = rung.global_rows();
    let owned_keys = OWNED_KEY_BASE..OWNED_KEY_BASE + OWNED_SLICE_ROWS;
    let global_keys = GLOBAL_KEY_BASE..GLOBAL_KEY_BASE + global_rows;

    let mut observed_owned = 0u64;
    let mut observed_global = 0u64;

    for row in rows {
        let (expected_owner, seen) = if owned_keys.contains(&row.entity_uuid) {
            (owner, &mut observed_owned)
        } else if global_keys.contains(&row.entity_uuid) {
            ensure!(
                role == RunRole::Control,
                "entity_owner_sender_view returned unrelated global row entity_uuid={} at \
                 {global_rows} global rows — security gate violated",
                row.entity_uuid,
            );
            (global, &mut observed_global)
        } else {
            bail!(
                "row entity_uuid={} lies outside both preregistered key ranges at {global_rows} \
                 global rows; nothing else was ever seeded",
                row.entity_uuid,
            );
        };
        ensure!(
            row.owner == expected_owner,
            "row entity_uuid={} is owned by {:?}, not the identity its key range was seeded for",
            row.entity_uuid,
            row.owner,
        );
        ensure!(
            row.record == PILOT_ROW_PAYLOAD,
            "row entity_uuid={} carries an unexpected payload; every seeded and measured row uses \
             the one fixed payload so width never confounds the comparison",
            row.entity_uuid,
        );
        *seen += 1;
    }

    ensure!(
        observed_owned == OWNED_SLICE_ROWS,
        "the observed result set holds {observed_owned} of the measured identity's {OWNED_SLICE_ROWS} \
         owned rows at {global_rows} global rows",
    );
    // The Arm's view is scoped to the sender, so it must show none of the global slice; the Control
    // subscribes to the base table, so it must show all of it. This is the read-set behavior the
    // candidate exists to measure, asserted rather than assumed.
    let expected_global = match role {
        RunRole::Arm => 0,
        RunRole::Control => global_rows,
    };
    ensure!(
        observed_global == expected_global,
        "the observed result set holds {observed_global} unrelated global rows at {global_rows} \
         global rows, expected {expected_global} for the {role:?}",
    );
    Ok(())
}

/// The disposition of an attempt that failed before or during its measured window, retaining
/// whatever rungs had already completed.
fn failed(kind: FailureKind, rungs: Vec<RungEvidence>, error: &Error) -> AttemptOutcome {
    let partial = PartialEvidence::sealed(rungs)
        .expect("a failure can only occur before the ladder's final rung completes");
    AttemptOutcome::Failed {
        kind,
        partial,
        diagnostic: DiagnosticArtifact::of_error(error),
    }
}

/// The disposition of an attempt that never reached a measurable state.
fn provision_failure(error: &Error) -> AttemptOutcome {
    failed(FailureKind::Provision, Vec::new(), error)
}

/// The diagnostic for a set of release failures, or `None` when every capability was provably
/// released. Aggregating rather than reporting only the first keeps a teardown failure from hiding
/// behind a disconnect failure — and is why the recorded reason names the failing *release* rather
/// than any one unreleased resource.
fn release_failure(release: Vec<Error>) -> Option<DiagnosticArtifact> {
    if release.is_empty() {
        None
    } else {
        Some(DiagnosticArtifact::of_error(&into_error(release)))
    }
}
