//! The calibration-Pilot driver for the fresh-server campaign (spec c33f2e51).

use std::path::Path;

use anyhow::{Context, Error, Result};
use spacetimedb_sdk::Identity;

use crate::client::connected_client::ConnectedClient;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::verified_module_artifact::VerifiedModuleArtifact;
use crate::observation::output_path::OutputPath;
use crate::provision::run_resources::RunResources;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;
use crate::view_read_set_campaign::attempt_inventory::AttemptInventory;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;
use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::composition_validation::composition_transition_expectation::CompositionTransitionExpectation;
use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;
use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::passed_environment_gate::PassedEnvironmentGate;
use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

/// A classified failure inside one attempt's measured window, carrying everything the terminal
/// record needs.
///
/// Copies the Pilot driver's private `MeasuredFailure` and adds the two things this protocol's
/// terminal record requires and that campaign's did not: the driver-stated [`FailureStage`], which
/// no cause can supply, and the execution-order channel prefix already completed, which
/// [`PartialEvidence`](super::partial_evidence::PartialEvidence) retains.
struct MeasuredFailure {
    kind: FailureKind,
    stage: FailureStage,
    measured: Vec<ChannelEvidence>,
    error: Error,
}

/// Everything one attempt's measured window produced when it ran to completion.
///
/// A private struct rather than a three-tuple because the two row sets are the same type and would
/// otherwise be distinguished only by position — and swapping them would judge the seeded state
/// against the after-batch expectation, which is exactly the near-miss
/// [`CompositionTransitionExpectation`](super::composition_validation::composition_transition_expectation::CompositionTransitionExpectation)
/// exists to prevent.
struct MeasuredAttempt {
    channels: Vec<ChannelEvidence>,
    before: ObservedRowSet,
    after: ObservedRowSet,
}

/// The outcome of acquiring one attempt's fresh server.
///
/// `Err` from the stage that returns this is reserved for a ledger persist failure alone — the
/// Pilot driver's convention — so a provisioning failure is a recorded disposition rather than the
/// campaign's error. The two variants are what the caller must distinguish anyway:
/// [`Self::Provisioned`] means a `Provisioned` line exists and
/// [`FailureStage::published`](super::failure_stage::FailureStage::published) must be true for any
/// later failure, while [`Self::FailedBeforePublish`] means no instance was created and none should
/// have been.
enum Provisioning {
    /// Published, provenance appended, and both capabilities still owned by the caller.
    Provisioned {
        resources: RunResources,
        artifact: VerifiedModuleArtifact,
    },
    /// Terminated before publication, having released whatever it had acquired.
    FailedBeforePublish {
        kind: FailureKind,
        diagnostic: DiagnosticArtifact,
    },
}

/// Run the whole calibration Pilot of the fresh-server campaign: freeze the inventory, create the
/// ledger at `output`, and execute every predeclared attempt in the frozen order, each on its own
/// fresh isolated server.
///
/// **Typed inputs, and one that is deliberately absent.** There is no `seed` parameter. The Pilot
/// this replaces took its block order from a CLI argument; this campaign's order derives from the
/// frozen [`CAMPAIGN_SEED`](super::campaign_params::CAMPAIGN_SEED), so
/// [`AttemptInventory::frozen`] takes no seed and no run-time input can produce a different
/// preregistration. `artifacts` is required because composition validation retains its observed row
/// sets as content-addressed files: without a directory there is nowhere for
/// [`ObservedRowSet::persisted`] to write, and a finding pointing at nothing would be unauditable.
///
/// **Phase boundary.** The pure [`schedule_retry`] and [`validate_final_composition`], the four
/// recording adapters, and [`settle`] are implemented; every other body in this file is still an
/// explicit `todo!()`. What is fixed for the rest is the stage decomposition, the typed inputs and
/// outputs of each stage, and the ordering discipline the stubs describe.
pub(crate) fn view_read_set_campaign_pilot(
    listen: ListenAddress,
    module_wasm: &Path,
    output: &OutputPath,
    artifacts: &Path,
) -> Result<()> {
    let _ = (listen, module_wasm, output, artifacts);
    todo!(
        "Phase 2: freeze the inventory with AttemptInventory::frozen and resolve \
         CampaignProvenance::resolved before creating anything; create the ledger with \
         CampaignSink::create; then run the campaign body and finalize the sink on every path — \
         including a failure of the very first write — surfacing both failures rather than letting \
         either hide the other, exactly as entity_owner_sender_view_pilot does. Nothing may `?` \
         past the sink's creation, since CampaignSink has no Drop backstop"
    )
}

/// Record the frozen order and campaign pins, then execute every predeclared attempt.
///
/// Split from the entrypoint so the opening `Inventory` write is inside the region its caller
/// finalizes: a persist failure on the very first line must still reach [`CampaignSink::finalize`].
fn run_campaign(
    sink: &mut CampaignSink,
    inventory: &AttemptInventory,
    provenance: &CampaignProvenance,
    listen: ListenAddress,
    module_wasm: &Path,
    artifacts: &Path,
) -> Result<()> {
    let _ = (sink, inventory, provenance, listen, module_wasm, artifacts);
    todo!("Phase 2: record_inventory, then run_inventory over the frozen order")
}

/// **Stage 1 — inventory write.** Append the frozen inventory and campaign provenance as the
/// ledger's first line, before any attempt executes.
///
/// This is what makes "report generation fails on missing or duplicate planned identities"
/// checkable: the order the evidence will be interpreted against is on disk before any evidence
/// exists, so a reader can name every predeclared slot even if the run dies before reaching it.
///
/// Both payloads are cloned into the owned record — the cost of a record vocabulary that is also
/// reconciliation's input — so the caller keeps the inventory it is about to walk. That this line
/// comes first is [`run_campaign`]'s doing, not this adapter's.
fn record_inventory(
    sink: &mut CampaignSink,
    inventory: &AttemptInventory,
    provenance: &CampaignProvenance,
) -> Result<()> {
    sink.append(CampaignRecord::Inventory {
        inventory: inventory.clone(),
        provenance: provenance.clone(),
    })
    .context("recording the campaign inventory")
    .map(|_seq| ())
}

/// Execute every frozen logical slot in order, appending exactly one terminal record per attempt
/// that runs, and scheduling the one permitted retry where a slot earned it.
///
/// **What the frozen inventory does and does not contain.** It predeclares *originals only*: it is
/// built before execution, and a retry identity is derived from a terminal outcome that has not
/// happened yet. So this walks originals, and a retry is an extra attempt run alongside the slot
/// that earned it — never a slot this loop was going to reach anyway.
///
/// Copies the Pilot's two stop conditions, which differ because the ledger's own health differs: a
/// **persist failure** poisons the sink so no truthful record about it could be appended, and
/// propagates; a **release failure** leaves the ledger healthy, so every untouched remaining slot is
/// recorded [`NotRun`](super::attempt_outcome::AttemptOutcome::NotRun) before stopping.
fn run_inventory(
    sink: &mut CampaignSink,
    inventory: &AttemptInventory,
    listen: ListenAddress,
    module_wasm: &Path,
    artifacts: &Path,
) -> Result<()> {
    let _ = (sink, inventory, listen, module_wasm, artifacts);
    todo!(
        "Phase 2: walk AttemptInventory::attempts in the frozen order, running each through \
         run_attempt; ask schedule_retry for each returned terminal record and, when it yields a \
         retry identity, run that attempt immediately after its original so a slot's two attempts \
         stay adjacent. On an unreleased-capability diagnostic, record NotRun with \
         NotRunReason::PriorAttemptReleaseFailed for every strictly later *frozen* identity and \
         stop loudly. A retry that was scheduled but never launched receives nothing: it is not a \
         predeclared slot, so it simply never becomes an identity, and reconciliation permits at \
         most one retry per slot without ever requiring one merely because the original was \
         eligible"
    )
}

/// Execute one attempt — a frozen logical slot's original, or a retry identity derived from one:
/// gate it, provision it, measure it, and settle it.
///
/// `Err` is reserved for a ledger persist failure, following the Pilot's convention; the returned
/// [`TerminalAttemptRecord`] is the disposition actually appended, and the
/// [`DiagnosticArtifact`] is present only when the attempt's capabilities were not provably
/// released.
fn run_attempt(
    sink: &mut CampaignSink,
    listen: ListenAddress,
    module_wasm: &Path,
    artifacts: &Path,
    attempt: AttemptKey,
) -> Result<(TerminalAttemptRecord, Option<DiagnosticArtifact>)> {
    let _ = (sink, listen, module_wasm, artifacts, attempt);
    todo!(
        "Phase 2, in this order, which is itself the contract:

         1. preflight_gate, since it is prospective and ends before launch. On \
         FailedEnvironmentGate::refused, settle immediately with AttemptOutcome::PreflightRejected \
         — nothing acquired, no clearance appended, and no post-attempt reading, because nothing \
         was measured.

         2. On a passing gate, record_preflight_cleared *before any provisioning or launch*, so the \
         ledger shows the clearance preceding the instance it cleared. A persist failure here stops \
         the attempt before launch, having acquired nothing.

         3. provision_and_record; on Provisioning::FailedBeforePublish settle with \
         AttemptOutcome::Failed at FailureStage::BeforePublish.

         4. With the instance live, connect the measured subscriber, seed the owned and global \
         slices, subscribe this role to its target, then measure_attempt and \
         validate_final_composition; seal ScalePointEvidence and EvidenceArtifact::for_attempt on \
         success.

         5. The moment the measured window ends — success or failure — call observe_environment \
         immediately, before building the terminal record, before persisting anything, and before \
         releasing any capability, so neither ledger latency nor teardown load can perturb the \
         reading the spec asks for 'immediately after'. Capture it for every measured attempt \
         (Complete, and Failed at MeasuredSampleBoundary::AfterFirst) and for no other, then carry \
         it into settle, which appends its line after the Terminal line.

         6. If that observation itself fails, the measured outcome does not change. A post-attempt \
         reading is supporting diagnostics that never invalidates evidence, so a Complete stays \
         Complete and a Failed keeps its own kind and stage: rewriting either would let a \
         retrospective diagnostic decide a measured disposition, which the contract forbids. Carry \
         the error into settle as Some(Err(_)) rather than converting it into an outcome.

         Every exit routes through settle, so the disposition is durable while the capabilities \
         that produced it are still owned"
    )
}

/// **Stage 2 — two-sample preflight.** Take the frozen prospective gate's two readings and pair
/// them.
///
/// Prospective by construction: this runs immediately before launch and decides whether an attempt
/// happens at all, so it can never invalidate a measurement.
fn preflight_gate() -> Result<EnvironmentGateEvidence> {
    todo!(
        "Phase 2: take the first sample, initiate a monotonic wait of \
         ENVIRONMENT_SAMPLE_SEPARATION_NANOS, take the second, and pair them with \
         EnvironmentGateEvidence::paired against the host's logical CPU count. Both offsets come \
         from one monotonic origin, so scheduler delay can only make the recorded interval longer \
         than sixty seconds, never shorter — which is why `paired` checks `>=`"
    )
}

/// Read the four host quantities the gate and the post-attempt diagnostic are both made of.
///
/// [`EnvironmentSample::observed`] is infallible because none of its fields has a forbidden value;
/// parsing `/proc`, which genuinely can fail, is this function's job and fails here.
fn observe_environment(offset_nanos: u64) -> Result<EnvironmentSample> {
    let _ = offset_nanos;
    todo!(
        "Phase 2: read /proc/loadavg, /proc/meminfo MemAvailable, /proc/vmstat pswpout, and \
         /proc/pressure/memory full avg60; convert load and PSI to hundredths as integers rather \
         than through a float, since hundredths are the exact reported resolution and the gate's \
         comparisons must be exact; convert pswpout pages to bytes with the page size; then hand \
         them to EnvironmentSample::observed at this offset"
    )
}

/// **Stage 2b — preflight clearance append.** Record the readings that cleared this attempt to
/// launch, before it launches.
///
/// A separate stage from the gate itself so the ordering is visible rather than buried: the
/// clearance line must precede the `Provisioned` line it cleared, and reconciliation requires
/// exactly one for every launched attempt. A refusal appends nothing here — it is already carried,
/// as a [`FailedEnvironmentGate`](super::failed_environment_gate::FailedEnvironmentGate), inside the
/// terminal record, and a second copy of a derived verdict could disagree with it.
fn record_preflight_cleared(
    sink: &mut CampaignSink,
    attempt: AttemptKey,
    gate: PassedEnvironmentGate,
) -> Result<()> {
    sink.append(CampaignRecord::PreflightCleared { attempt, gate })
        .context("recording the preflight clearance")
        .map(|_seq| ())
}

/// **Stage 3 — provision, publish, and append provenance.** Acquire one attempt's fresh isolated
/// server and record which instance it will measure against, before anything connects.
///
/// The provenance line is appended *after* publish succeeds and *before* any connection, so no
/// evidence can exist that is not already joinable to its runtime and its freshly published
/// database.
fn provision_and_record(
    sink: &mut CampaignSink,
    distribution: &VerifiedDistribution,
    listen: ListenAddress,
    module_wasm: &Path,
    attempt: AttemptKey,
) -> Result<Provisioning> {
    let _ = (sink, distribution, listen, module_wasm, attempt);
    todo!(
        "Phase 2: stage the module with StagedModuleWasm::load against MODULE_WASM_SHA256, start \
         the pinned standalone with RunningPinnedServer::start, and publish. Release whatever was \
         acquired before returning Provisioning::FailedBeforePublish, since the caller holds \
         nothing on that path. On success bundle RunResources::new, snapshot \
         AttemptProvenance::observed, and append CampaignRecord::Provisioned; a persist failure \
         there is the ledger poison this function's Err is reserved for, and the caller must \
         release the live resources"
    )
}

/// **Stage 4 — measurement.** Run all four channels in the frozen execution order, capturing the
/// composition observations the transition will be judged against.
///
/// The before-observation is taken after E4 and before E2's first measured write, which is where the
/// spec's "before E2, every seeded row has the seeded payload" holds; the after-observation is taken
/// once E1's batch has confirmed, which is the final state because E1 runs last.
fn measure_attempt(
    client: &ConnectedClient,
    attempt: AttemptKey,
    artifacts: &Path,
) -> std::result::Result<MeasuredAttempt, MeasuredFailure> {
    let _ = (client, attempt, artifacts);
    todo!(
        "Phase 2: walk MeasurementChannel::EXECUTION_ORDER — E3, E4, E2, E1 — through \
         measure_channel, pushing each ChannelEvidence onto the prefix so a failure can return \
         whatever had completed. Retain the before-observation with ObservedRowSet::persisted after \
         E4 and before E2, and the after-observation once E1 confirms. Every failure carries the \
         FailureStage the driver knows: AfterPublishBeforeFirstSample until the first measured \
         sample confirms, AfterFirstSample from then on"
    )
}

/// Measure one channel at this attempt's scale point, reducing its own samples to its own statistic.
fn measure_channel(
    client: &ConnectedClient,
    attempt: AttemptKey,
    channel: MeasurementChannel,
) -> std::result::Result<ChannelEvidence, MeasuredFailure> {
    let _ = (client, attempt, channel);
    todo!(
        "Phase 2: total-match the channel. ColdSubscriptionApplyTime times the initial snapshot of \
         the fresh connection, measured exactly once because a second subscription is no longer \
         cold; ReconnectApplyTime times a token-preserving reconnect and resubscribe; \
         PacedVisibleApplyLatency takes CHANNEL_SAMPLE_COUNT samples with one outstanding write at \
         a time, each stopped when the change is observable in the subscriber cache, with \
         PACED_SAMPLE_DELAY_MS between completed samples; SaturatedQueueGrowthPerWrite issues the \
         same count back-to-back without awaiting, recording issue and confirmation offsets from a \
         common origin. The two write-issuing channels drive MutationSchedule::of(channel), whose \
         target_key and payload derivations checkpoint 1ffbbe9a implemented, through \
         ConnectedClient::update_entity_owner — the owner-preserving update path that checkpoint \
         added together with the module's own reducer. Each ChannelEvidence constructor performs \
         its own reduction, so no statistic is computed here"
    )
}

/// **Stage 5 — final composition transition.** Judge the two retained observations against the one
/// phase-matched expectation derived from this attempt's identity.
///
/// A two-call join with no rule of its own. The expectation is minted *here* rather than passed in,
/// from this attempt's own scale, role, and the two identities, so the transition a finding is
/// checked against is necessarily the transition this attempt's identity requires. Which mutation
/// witnesses the batch is not a parameter either: [`ValidatedComposition::validate`] derives it from
/// the bound after-phase schedule, so a caller cannot claim a finding about the final write while
/// evidencing an earlier one.
///
/// **Why every refusal is one kind and one stage.** A composition refusal means the retained rows do
/// not hold what this role must observe — the Arm carrying any foreign row is the candidate's answer
/// about its read set, not an accident of the run — so it is
/// [`SemanticsOrSecurity`](FailureKind::SemanticsOrSecurity) rather than an operational fault. It is
/// [`AfterFirstSample`](FailureStage::AfterFirstSample) because the check runs only once the
/// saturated batch has confirmed, which is necessarily after the first measured sample.
///
/// The measured channel prefix is returned on both paths — moved through on success, retained in the
/// failure on refusal — because a composition refusal does not unmeasure the channels that ran, and
/// their evidence is what a partial record is made of.
fn validate_final_composition(
    attempt: AttemptKey,
    measured: MeasuredAttempt,
    owned_owner: Identity,
    foreign_owner: Identity,
) -> std::result::Result<(Vec<ChannelEvidence>, ValidatedComposition), MeasuredFailure> {
    let expected = CompositionTransitionExpectation::required_final(
        attempt.scale(),
        attempt.role(),
        owned_owner,
        foreign_owner,
    );
    let MeasuredAttempt {
        channels,
        before,
        after,
    } = measured;

    match ValidatedComposition::validate(expected, before, after) {
        Ok(composition) => Ok((channels, composition)),
        Err(error) => Err(MeasuredFailure {
            kind: FailureKind::SemanticsOrSecurity,
            stage: FailureStage::AfterFirstSample,
            measured: channels,
            error,
        }),
    }
}

/// **Stage 6a — terminal append.** Append one attempt's single terminal record.
///
/// Borrows rather than consumes: [`settle`] returns the same record to its caller, which needs it
/// for [`schedule_retry`], so consuming here would force that caller to rebuild an identical record
/// — a second construction that could differ from the one actually on disk.
///
/// The assigned sequence is not read back, here or in any of the other three appends: the ledger is
/// append-only and nothing in this driver addresses a record by position.
fn record_terminal(sink: &mut CampaignSink, record: &TerminalAttemptRecord) -> Result<()> {
    sink.append(CampaignRecord::Terminal {
        record: record.clone(),
    })
    .context("recording the attempt terminal outcome")
    .map(|_seq| ())
}

/// **Stage 6b — post-attempt append.** Append the host reading taken immediately after a measured
/// attempt.
///
/// Supporting diagnostics only. It never invalidates evidence, no retry criterion may reference it,
/// and reconciliation requires exactly one for every measured attempt and none for any other — so
/// omitting it for a Complete or `AfterFirst` outcome is a rejection, not a tidy ledger.
///
/// Reconciliation requires this line at exactly the terminal line's next sequence. That adjacency is
/// [`settle`]'s doing — it calls [`record_terminal`] and then this with nothing between, on a sink
/// whose counter advances exactly once per successful append — and cannot be checked from inside a
/// single append.
fn record_post_attempt(
    sink: &mut CampaignSink,
    attempt: AttemptKey,
    sample: EnvironmentSample,
) -> Result<()> {
    sink.append(CampaignRecord::PostAttemptEnvironment { attempt, sample })
        .context("recording the post-attempt environment")
        .map(|_seq| ())
}

/// **Stage 7 — retry scheduling.** The identity of the one permitted retry of this record's logical
/// slot, when the spec's rule grants it.
///
/// Takes the bound [`TerminalAttemptRecord`] rather than a key and an outcome, so the retry ordinal
/// consulted is necessarily the ordinal of the attempt that produced the outcome consulted.
///
/// **The split of responsibility, which is the whole design.** Whether this slot has earned another
/// attempt is an *outcome* question, and it is already answered whole by
/// [`TerminalAttemptRecord::retry_eligibility`] — including the protocol's global cap of one retry
/// per slot, which that function applies to every outcome alike. What the identity of that retry
/// *is* is a question about the key, answered by [`AttemptKey::next_retry`]. Neither is restated
/// here, so this function is exactly the join of the two and has no rule of its own to drift.
///
/// The three eligibility states are matched by name rather than compared against `Eligible`, so a
/// fourth must state its own scheduling disposition instead of falling into a catch-all.
///
/// **This promises nothing about *when*.** A returned identity says a retry is authorized, not that
/// it has been placed; [`run_inventory`] remains responsible for running it immediately after the
/// original it belongs to, which is a property of the driver's schedule and deliberately not one
/// reconciliation re-derives from the ledger.
fn schedule_retry(record: &TerminalAttemptRecord) -> Option<AttemptKey> {
    match record.retry_eligibility() {
        // Nothing to schedule, for two different reasons that both end here: the slot was refused
        // another attempt, or its outcome raises no retry question at all.
        RetryEligibility::Ineligible | RetryEligibility::None => None,
        RetryEligibility::Eligible => Some(record.key().next_retry().expect(
            "retry eligibility applies the one-retry ordinal cap globally, so a record it reports \
             as eligible is always at the original ordinal and always has a next retry",
        )),
    }
}

/// **Stage 8 — cleanup.** Close out one attempt while its capabilities are still owned: write its
/// terminal record, then release, then report.
///
/// Copies the Pilot's `settle` discipline exactly, and the executable order is the invariant:
/// terminal first, so the disposition is durable before any capability is handed back — skipped only
/// when the outcome is already a persist failure, since a further write would be dishonest rather
/// than merely futile; `release` always runs, and is a closure precisely so it cannot be evaluated
/// before the write; and a ledger error leads, with release failures aggregated after it.
///
/// `post_attempt` is the reading already taken, at the instant the measured window ended — it is
/// *carried* here, never taken here, because a sample observed after a terminal persist and a
/// teardown would describe the harness's own cleanup rather than the attempt's environment. Its
/// three states are the three that exist, and none of them can alter the outcome:
///
/// - `None` — this attempt measured nothing, so no post-attempt line is required or permitted;
/// - `Some(Ok(sample))` — measured, and the reading was taken;
/// - `Some(Err(error))` — measured, and the reading itself failed. The measured disposition still
///   stands, because a retrospective diagnostic may not decide it, but that error becomes the
///   primary and stops the campaign: the ledger now permanently lacks a line reconciliation
///   requires for a measured attempt, so continuing would append later attempts under a contract
///   this ledger can no longer satisfy. What is lost is the campaign, not the attempt — the
///   evidence already written stays on disk and stays valid.
///
/// A failed Terminal append skips the post-attempt line rather than offering it to a sink that has
/// just poisoned itself. And nothing here checks `post_attempt` against the outcome: whether a
/// measured attempt carries a reading is [`run_attempt`]'s to get right and reconciliation's to
/// reject, and re-deriving it here would be a second copy of that rule.
fn settle(
    sink: &mut CampaignSink,
    record: Result<TerminalAttemptRecord>,
    post_attempt: Option<Result<EnvironmentSample>>,
    release: impl FnOnce() -> Vec<Error>,
) -> Result<(TerminalAttemptRecord, Option<DiagnosticArtifact>)> {
    let settled = match record {
        Err(primary) => Err(primary),
        Ok(record) => match record_terminal(sink, &record) {
            Err(primary) => Err(primary),
            Ok(()) => match post_attempt {
                None => Ok(record),
                Some(Err(primary)) => Err(primary),
                Some(Ok(sample)) => {
                    record_post_attempt(sink, record.key(), sample).map(|()| record)
                }
            },
        },
    };

    let mut release_errors = release();

    match settled {
        Ok(record) => Ok((record, release_failure(release_errors))),
        Err(primary) => {
            let mut errors = vec![primary];
            errors.append(&mut release_errors);
            Err(into_error(errors))
        }
    }
}

/// The diagnostic for a set of release failures, or `None` when every capability was provably
/// released.
///
/// The Pilot's helper, copied rather than shared: it belongs to that campaign's driver, and the two
/// drivers are deliberately separate modules. Aggregating rather than reporting only the first keeps
/// a teardown failure from hiding behind a disconnect failure.
fn release_failure(release: Vec<Error>) -> Option<DiagnosticArtifact> {
    if release.is_empty() {
        None
    } else {
        Some(DiagnosticArtifact::of_error(&into_error(release)))
    }
}

// A child of this module, which is what lets it reach the private `schedule_retry`. Unlike the
// sealed minting clusters elsewhere in this campaign, nothing here is protected by that privacy:
// `schedule_retry` is a pure function over a record anyone in the crate can already build, so a
// child test module can forge nothing it could not forge from outside.
#[cfg(test)]
mod tests;
