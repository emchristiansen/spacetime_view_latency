//! The calibration-Pilot driver for the fresh-server campaign (spec c33f2e51).

use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, ensure, Context, Error, Result};
use spacetimedb_sdk::Identity;

use crate::client::connected_client::ConnectedClient;
use crate::client::measured_step_failure::MeasuredStepFailure;
use crate::client::reconnect_failure::ReconnectFailure;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::verified_module_artifact::VerifiedModuleArtifact;
use crate::module_artifact::bindings::SubscriptionHandle;
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
use crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory;
use crate::view_read_set_campaign::composition_validation::composition_transition_expectation::CompositionTransitionExpectation;
use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;
use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::infrastructure_phase::InfrastructurePhase;
use crate::view_read_set_campaign::measured_sample_boundary::MeasuredSampleBoundary;
use crate::view_read_set_campaign::measured_target::MeasuredTarget;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;
use crate::view_read_set_campaign::passed_environment_gate::PassedEnvironmentGate;
use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::scale_point_evidence::CHANNEL_COUNT;
use crate::view_read_set_campaign::subscriber_delivery::delivery_counters::DeliveryCounters;
use crate::view_read_set_campaign::subscriber_delivery::subscriber_delivery_evidence::SubscriberDeliveryEvidence;
use crate::view_read_set_campaign::subscriber_delivery::subscriber_delivery_meter::SubscriberDeliveryMeter;
use crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counter::SubscriberRowCounter;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

/// The kernel sources of the four gate quantities, and the exact keys and shapes read out of them.
///
/// `/proc/pressure/memory` exists only on a `CONFIG_PSI` kernel; its absence fails the read rather
/// than substituting a value.
const PROC_LOADAVG: &str = "/proc/loadavg";
const PROC_MEMINFO: &str = "/proc/meminfo";
const PROC_VMSTAT: &str = "/proc/vmstat";
const PROC_PRESSURE_MEMORY: &str = "/proc/pressure/memory";

/// Three load averages, `running/total`, and the last pid.
const PROC_LOADAVG_FIELDS: usize = 5;
const PROC_LOADAVG_TASKS_FIELD: usize = 3;
const PROC_LOADAVG_LAST_PID_FIELD: usize = 4;
const LOADAVG_TASK_SEPARATOR: char = '/';
/// The whole first field, colon included — `meminfo` prints the key and its value whitespace-separated.
const MEMINFO_AVAILABLE_KEY: &str = "MemAvailable:";
/// Key, value, unit.
const MEMINFO_FIELDS: usize = 3;
/// Spelled `kB`, meaning kibibytes — see [`parse_available_ram_bytes`].
const MEMINFO_KIBIBYTE_UNIT: &str = "kB";
const BYTES_PER_KIBIBYTE: u64 = 1_024;
const VMSTAT_SWAP_OUT_KEY: &str = "pswpout";
/// Key and value.
const VMSTAT_FIELDS: usize = 2;
const PRESSURE_FULL_KEY: &str = "full";
/// Kind, three averages, cumulative total.
const PRESSURE_FIELDS: usize = 5;
const PRESSURE_AVG10_FIELD: usize = 1;
const PRESSURE_AVG60_FIELD: usize = 2;
const PRESSURE_AVG300_FIELD: usize = 3;
const PRESSURE_TOTAL_FIELD: usize = 4;
const PRESSURE_AVG10_KEY: &str = "avg10=";
const PRESSURE_AVG60_KEY: &str = "avg60=";
const PRESSURE_AVG300_KEY: &str = "avg300=";
const PRESSURE_TOTAL_KEY: &str = "total=";
const DECIMAL_POINT: char = '.';
/// Decimal places both `/proc/loadavg` and `/proc/pressure/memory` print — the exact resolution the
/// samples are stored in.
const CENTI_FRACTION_DIGITS: usize = 2;
const CENTI_PER_UNIT: u64 = 100;

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

/// What one measured window left for its caller to release.
///
/// Returned outside the measurement's `Result` so the obligation arrives as a value to destructure
/// rather than something to remember behind a `?`. That makes the ownership explicit; release-always
/// itself belongs to [`run_attempt`], still a `todo!()`, where the contract is written.
///
/// All three states are reachable. The window normally ends connected; the reconnect is the
/// exception, releasing the old connection before initiating the new one, so a failed reconnect can
/// leave nothing — which is why no `Option` or dummy client is needed anywhere here.
///
/// [`Self::ReleasedWithError`] is not "something may still be running": every client release path
/// joins the message-processing thread. It means the connection ended *abnormally*, and is still
/// campaign-terminal through [`settle`]'s release aggregation, but is a cleanup fault kept distinct
/// from the attempt's cause.
enum MeasuredRelease {
    /// Still connected, and now the caller's to disconnect exactly once.
    Connected(ConnectedClient),
    /// No client exists and none is outstanding.
    Released,
    /// No client exists, but the release ended abnormally. A resource-release failure, never the
    /// attempt's measured cause.
    ReleasedWithError(Error),
}

/// The measured subscriber, and its retained subscription once one exists.
///
/// A closed two-state machine rather than a client beside an `Option<SubscriptionHandle>`: exactly
/// one channel creates the subscription (the cold subscription, which the frozen order runs first
/// because nothing else may have subscribed yet) and exactly one replaces it (the reconnect).
///
/// **Exactly one handle, the live connection's.** Pinned SDK 2.7.0 does not mark an applied
/// subscription inactive when its connection ends — `SubscriptionManager::on_disconnect` is a
/// deliberate no-op — so a handle retained across the reconnect would report itself active forever.
enum MeasuredSession {
    /// Connected, with nothing subscribed on this connection yet.
    Unsubscribed { client: ConnectedClient },
    /// Connected, with exactly one retained subscription on *this* connection.
    Subscribed {
        client: ConnectedClient,
        subscription: SubscriptionHandle,
    },
}

impl MeasuredSession {
    /// The measured client, for a step that only reads through it.
    fn client(&self) -> &ConnectedClient {
        match self {
            Self::Unsubscribed { client } | Self::Subscribed { client, .. } => client,
        }
    }

    /// The concrete handles the delivery evidence derives its active count from — never a count.
    fn retained_subscriptions(&self) -> &[SubscriptionHandle] {
        match self {
            Self::Unsubscribed { .. } => &[],
            Self::Subscribed { subscription, .. } => std::slice::from_ref(subscription),
        }
    }

    /// Hand the client back to the caller for release. Total: every session holds one.
    fn into_release(self) -> MeasuredRelease {
        match self {
            Self::Unsubscribed { client } | Self::Subscribed { client, .. } => {
                MeasuredRelease::Connected(client)
            }
        }
    }
}

/// One channel's failure together with what the caller cannot recover from the prefix alone.
///
/// The release, because the channels disagree: three fail with the client still connected, and only
/// the reconnect can fail having released it. And [`Self::observed`], because a channel can observe
/// its sample and *then* fail at its reduction. [`measure_attempt`] adds the completed prefix.
struct ChannelFailure {
    release: MeasuredRelease,
    /// Whether this channel took an observation the completed prefix does not contain —
    /// [`AfterFirst`](MeasuredSampleBoundary::AfterFirst) exactly when its reduction refused the
    /// statistic, [`BeforeFirst`](MeasuredSampleBoundary::BeforeFirst) for an operational failure.
    observed: MeasuredSampleBoundary,
    kind: FailureKind,
    error: Error,
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
/// recording adapters, [`settle`], and the measurement stage — [`measure_attempt`] and
/// [`measure_channel`] — are implemented; every other body in this file is still an explicit
/// `todo!()`. What remains is orchestration and the two `/proc`-reading gate stages: this entrypoint,
/// [`run_campaign`], [`run_inventory`], [`run_attempt`], [`preflight_gate`], [`observe_environment`],
/// and [`provision_and_record`]. What is fixed for those is the stage decomposition, the typed inputs
/// and outputs of each stage, and the ordering discipline the stubs describe.
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
/// [`DiagnosticArtifact`] is present only when the attempt's capabilities were not released cleanly.
///
/// **The release contract this stage owes [`measure_attempt`].** That stage hands back a
/// [`MeasuredRelease`] outside its `Result`. Bind the pair with a plain `let`, then honour all three
/// states inside [`settle`]'s closure, so every release happens after the terminal record is
/// durable:
///
/// - [`Connected`](MeasuredRelease::Connected) — disconnect it. `disconnect` consumes, so once, not
///   twice.
/// - [`Released`](MeasuredRelease::Released) — nothing; the resources teardown still runs.
/// - [`ReleasedWithError`](MeasuredRelease::ReleasedWithError) — push the carried error onto the
///   release errors, reaching the usual `PriorAttemptReleaseFailed` path.
///
/// Until this body exists that is a contract for its implementor, not a compiler-enforced property.
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

         4. With the instance live, connect the measured subscriber and seed the owned and global \
         slices, then hand the client to measure_attempt, which owns it for the whole measured \
         window. Do *not* subscribe here: measure_attempt registers the delivery callbacks and \
         issues the cold subscription itself, because E3 must be the first subscription this \
         connection ever makes and its meter must already be open. Bind its (release, result) pair \
         with a plain `let`, then validate_final_composition; on success seal ScalePointEvidence \
         with the returned delivery evidence and build EvidenceArtifact::for_attempt.

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
///
/// **Four reads, not one snapshot**: the kernel offers no combined atomic view of these files, so a
/// sample is four readings in quick succession.
///
/// Each parser below selects its line by exact whole-field key and validates that line whole — every
/// field, including the ones it does not return — so a changed format is an error instead of a
/// plausible wrong number.
fn observe_environment(offset_nanos: u64) -> Result<EnvironmentSample> {
    let loadavg = read_proc(PROC_LOADAVG)?;
    let meminfo = read_proc(PROC_MEMINFO)?;
    let vmstat = read_proc(PROC_VMSTAT)?;
    let pressure = read_proc(PROC_PRESSURE_MEMORY)?;

    let one_minute_load_centi =
        parse_one_minute_load_centi(&loadavg).with_context(|| format!("parsing {PROC_LOADAVG}"))?;
    let available_ram_bytes =
        parse_available_ram_bytes(&meminfo).with_context(|| format!("parsing {PROC_MEMINFO}"))?;
    let swap_out_pages =
        parse_swap_out_pages(&vmstat).with_context(|| format!("parsing {PROC_VMSTAT}"))?;
    let memory_psi_full_avg60_centi = parse_memory_psi_full_avg60_centi(&pressure)
        .with_context(|| format!("parsing {PROC_PRESSURE_MEMORY}"))?;

    let cumulative_swap_out_bytes = swap_out_pages
        .checked_mul(host_page_size_bytes()?)
        .with_context(|| {
            format!("converting {swap_out_pages} swapped-out pages to bytes overflowed u64")
        })?;

    Ok(EnvironmentSample::observed(
        offset_nanos,
        one_minute_load_centi,
        available_ram_bytes,
        cumulative_swap_out_bytes,
        memory_psi_full_avg60_centi,
    ))
}

/// Read one `/proc` file whole, naming it on failure.
fn read_proc(path: &str) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {path}"))
}

/// The host's memory page size in bytes — `pswpout` counts pages, the sample records bytes.
///
/// Queried at runtime rather than frozen as a constant, which would bake an unverified premise into
/// recorded evidence. `sysconf` reports an unsupported name as `-1`, and `0` is not a page size, so
/// both are refused rather than becoming a factor that zeroes every swap reading.
fn host_page_size_bytes() -> Result<u64> {
    // SAFETY: `sysconf` reads a static system parameter. It takes one `c_int` by value, borrows no
    // memory from this process, writes nothing, and has no precondition beyond a name the platform
    // defines — which `libc::_SC_PAGESIZE` is. Its only failure is the in-band `-1`, checked below.
    let raw = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    ensure!(raw > 0, "sysconf(_SC_PAGESIZE) reported {raw}");
    u64::try_from(raw).with_context(|| format!("page size {raw} does not fit u64"))
}

/// The one-minute load average from `/proc/loadavg`, in hundredths of a runnable task.
///
/// The whole line is validated, not just the field returned: three two-decimal averages, a
/// `running/total` pair of counts, and the last pid. A line that fails any of those is not
/// `/proc/loadavg`, and nothing about it places the one-minute average.
fn parse_one_minute_load_centi(loadavg: &str) -> Result<u64> {
    let line = sole_line(loadavg)?;
    let fields: Vec<&str> = line.split_whitespace().collect();
    ensure!(
        fields.len() == PROC_LOADAVG_FIELDS,
        "expected {PROC_LOADAVG_FIELDS} whitespace-separated fields, got {}: {line:?}",
        fields.len()
    );

    let one_minute = parse_two_decimal_centi(fields[0])
        .with_context(|| format!("reading the one-minute load average of {line:?}"))?;
    parse_two_decimal_centi(fields[1])
        .with_context(|| format!("reading the five-minute load average of {line:?}"))?;
    parse_two_decimal_centi(fields[2])
        .with_context(|| format!("reading the fifteen-minute load average of {line:?}"))?;

    let (running, total) = fields[PROC_LOADAVG_TASKS_FIELD]
        .split_once(LOADAVG_TASK_SEPARATOR)
        .with_context(|| {
            format!(
                "expected a `running{LOADAVG_TASK_SEPARATOR}total` task pair, got {:?}: {line:?}",
                fields[PROC_LOADAVG_TASKS_FIELD]
            )
        })?;
    parse_count(running).with_context(|| format!("reading the runnable task count of {line:?}"))?;
    parse_count(total).with_context(|| format!("reading the total task count of {line:?}"))?;
    parse_count(fields[PROC_LOADAVG_LAST_PID_FIELD])
        .with_context(|| format!("reading the last pid of {line:?}"))?;

    Ok(one_minute)
}

/// `MemAvailable` from `/proc/meminfo`, in bytes.
///
/// **The kernel's `kB` suffix means kibibytes**, so the multiplier is 1024; reading it literally
/// would understate available memory by 2.4%.
///
/// Selected by whole-field equality, not by prefix: a `MemAvailable:`-prefixed key like
/// `MemAvailable:Extra` would otherwise satisfy every later check and be read as this quantity.
fn parse_available_ram_bytes(meminfo: &str) -> Result<u64> {
    let line = sole_line_with_first_field(meminfo, MEMINFO_AVAILABLE_KEY)?;
    let fields: Vec<&str> = line.split_whitespace().collect();
    ensure!(
        fields.len() == MEMINFO_FIELDS,
        "expected `{MEMINFO_AVAILABLE_KEY} <value> {MEMINFO_KIBIBYTE_UNIT}`, got {line:?}"
    );
    ensure!(
        fields[MEMINFO_FIELDS - 1] == MEMINFO_KIBIBYTE_UNIT,
        "expected the unit {MEMINFO_KIBIBYTE_UNIT:?}, got {:?}: {line:?}",
        fields[MEMINFO_FIELDS - 1]
    );
    let kibibytes: u64 = fields[1]
        .parse()
        .with_context(|| format!("reading {:?} as a count of kibibytes", fields[1]))?;
    kibibytes
        .checked_mul(BYTES_PER_KIBIBYTE)
        .with_context(|| format!("converting {kibibytes} kibibytes to bytes overflowed u64"))
}

/// Cumulative pages swapped out since boot, from `/proc/vmstat`'s `pswpout` counter.
///
/// Matched on the whole first field, not a prefix: neighbouring `pswpin` shares five characters.
fn parse_swap_out_pages(vmstat: &str) -> Result<u64> {
    let line = sole_line_with_first_field(vmstat, VMSTAT_SWAP_OUT_KEY)?;
    let fields: Vec<&str> = line.split_whitespace().collect();
    ensure!(
        fields.len() == VMSTAT_FIELDS,
        "expected `{VMSTAT_SWAP_OUT_KEY} <pages>`, got {line:?}"
    );
    fields[1]
        .parse()
        .with_context(|| format!("reading {:?} as a count of swapped-out pages", fields[1]))
}

/// Memory pressure `full avg60` from `/proc/pressure/memory`, in hundredths of a percent.
///
/// The `full` line, which the frozen gate names, never `some`; both carry an identically spelled
/// `avg60`, so the line is selected before the field.
///
/// The whole line is validated, not just the field returned: each of the three averages carries its
/// own key and a two-decimal value, and the cumulative total carries its key and a count. So `avg60`
/// is placed by the line's proven shape rather than by trusting one position.
fn parse_memory_psi_full_avg60_centi(pressure: &str) -> Result<u64> {
    let line = sole_line_with_first_field(pressure, PRESSURE_FULL_KEY)?;
    let fields: Vec<&str> = line.split_whitespace().collect();
    ensure!(
        fields.len() == PRESSURE_FIELDS,
        "expected `{PRESSURE_FULL_KEY} {PRESSURE_AVG10_KEY}… {PRESSURE_AVG60_KEY}… \
         {PRESSURE_AVG300_KEY}… {PRESSURE_TOTAL_KEY}…`, got {line:?}"
    );

    parse_two_decimal_centi(keyed_field(
        &fields,
        PRESSURE_AVG10_FIELD,
        PRESSURE_AVG10_KEY,
        line,
    )?)
    .with_context(|| format!("reading the ten-second average of {line:?}"))?;
    let avg60 = parse_two_decimal_centi(keyed_field(
        &fields,
        PRESSURE_AVG60_FIELD,
        PRESSURE_AVG60_KEY,
        line,
    )?)
    .with_context(|| format!("reading the sixty-second average of {line:?}"))?;
    parse_two_decimal_centi(keyed_field(
        &fields,
        PRESSURE_AVG300_FIELD,
        PRESSURE_AVG300_KEY,
        line,
    )?)
    .with_context(|| format!("reading the three-hundred-second average of {line:?}"))?;
    parse_count(keyed_field(
        &fields,
        PRESSURE_TOTAL_FIELD,
        PRESSURE_TOTAL_KEY,
        line,
    )?)
    .with_context(|| format!("reading the cumulative stall total of {line:?}"))?;

    Ok(avg60)
}

/// The value of a `key=value` field at its fixed position, failing loud if it carries another key.
fn keyed_field<'a>(fields: &[&'a str], index: usize, key: &str, line: &str) -> Result<&'a str> {
    fields[index].strip_prefix(key).with_context(|| {
        format!(
            "expected field {index} to carry {key:?}, got {:?}: {line:?}",
            fields[index]
        )
    })
}

/// An unsigned count as the kernel prints it.
fn parse_count(field: &str) -> Result<u64> {
    field
        .parse()
        .with_context(|| format!("reading {field:?} as a count"))
}

/// Parse a fixed two-decimal-place kernel figure into exact hundredths, through integers rather
/// than a float — `"1.23"` via `f64` can land on `122.99999…`, and the gate compares exactly.
///
/// Exactly two fraction digits, not at most two: reading `"1.2"` as 12 hundredths rather than 120
/// would scale a gate term by ten.
fn parse_two_decimal_centi(field: &str) -> Result<u64> {
    let (whole, fraction) = field
        .split_once(DECIMAL_POINT)
        .with_context(|| format!("expected a decimal figure, got {field:?}"))?;
    ensure!(
        fraction.len() == CENTI_FRACTION_DIGITS,
        "expected exactly {CENTI_FRACTION_DIGITS} decimal places, got {:?}",
        field
    );
    let whole: u64 = whole
        .parse()
        .with_context(|| format!("reading {whole:?} as the whole part of {field:?}"))?;
    let fraction: u64 = fraction
        .parse()
        .with_context(|| format!("reading {fraction:?} as the fractional part of {field:?}"))?;
    whole
        .checked_mul(CENTI_PER_UNIT)
        .and_then(|centi| centi.checked_add(fraction))
        .with_context(|| format!("converting {field:?} to hundredths overflowed u64"))
}

/// The single line of a one-line `/proc` file, rejecting an empty or multi-line reading.
fn sole_line(contents: &str) -> Result<&str> {
    let mut lines = contents.lines();
    let line = lines.next().context("the file was empty")?;
    ensure!(
        lines.next().is_none(),
        "expected exactly one line, got {}",
        contents.lines().count()
    );
    Ok(line)
}

/// The one line whose first whitespace-separated field is exactly `key`.
fn sole_line_with_first_field<'a>(contents: &'a str, key: &str) -> Result<&'a str> {
    sole_line_matching(contents, key, |line| {
        line.split_whitespace().next() == Some(key)
    })
}

/// The one line satisfying `is_match`. Uniqueness is required rather than taking the first match, so
/// a file that grew a second line with the same key fails instead of silently choosing.
fn sole_line_matching<'a>(
    contents: &'a str,
    key: &str,
    is_match: impl Fn(&str) -> bool,
) -> Result<&'a str> {
    let mut matched = contents.lines().filter(|line| is_match(line));
    let line = matched
        .next()
        .with_context(|| format!("no line for {key:?}"))?;
    ensure!(
        matched.next().is_none(),
        "expected exactly one line for {key:?}, found more"
    );
    Ok(line)
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
/// composition observations the transition will be judged against and the delivery facts of the
/// window they ran in.
///
/// **Consumes the client and returns its release state outside the `Result`**, because which client
/// — if any — survives is measured here, not knowable by the caller. See [`MeasuredRelease`].
///
/// **Delivery evidence comes back beside the measurement, not inside it**, so
/// [`validate_final_composition`] stays pure and provable from retained artifacts. [`run_attempt`]
/// seals the two together.
///
/// **The order is the contract.** Callbacks registered and meter opened *before* the cold
/// subscription, so its initial snapshot counts as delivered rows. The before-observation after the
/// apply channels and before the first measured write, where "every seeded row has the seeded
/// payload" holds. After the saturated batch confirms, the cache is read **once**, and that one
/// snapshot both closes the window and becomes the after-phase artifact, so cardinality and retained
/// rows cannot describe different states.
///
/// Every failure's stage is stated in [`failure_stage`].
fn measure_attempt(
    client: ConnectedClient,
    attempt: AttemptKey,
    artifacts: &Path,
) -> (
    MeasuredRelease,
    std::result::Result<(MeasuredAttempt, SubscriberDeliveryEvidence), MeasuredFailure>,
) {
    let target = MeasuredTarget::of(attempt.role());
    let rows = Arc::new(SubscriberRowCounter::new());
    let mut channels: Vec<ChannelEvidence> = Vec::with_capacity(CHANNEL_COUNT);

    let directory = match AttemptArtifactDirectory::create(artifacts, attempt) {
        Ok(directory) => directory,
        Err(error) => {
            return measured_failure(
                MeasuredRelease::Connected(client),
                channels,
                MeasuredSampleBoundary::BeforeFirst,
                FailureKind::Infrastructure(InfrastructurePhase::ObservationRetention),
                error,
            )
        }
    };

    // Before the cold subscription, so its initial snapshot is counted as the delivery it is.
    client.register_delivery_callbacks(target, DeliveryCounters::of(&rows));

    let meter = match SubscriberDeliveryMeter::open(&client, rows.clone()) {
        Ok(meter) => meter,
        Err(error) => {
            return measured_failure(
                MeasuredRelease::Connected(client),
                channels,
                MeasuredSampleBoundary::BeforeFirst,
                // The measured window's own apparatus, which is what `Sample` covers besides the
                // writes themselves — "issuing a measured write, or sealing a channel's batch".
                // Whichever phase this is read as, the retry rule reaches the same answer, because
                // every infrastructure cause before the first sample classifies alike.
                FailureKind::Infrastructure(InfrastructurePhase::Sample),
                error,
            );
        }
    };

    // The frozen order's two halves, split by whether a channel issues measured writes rather than
    // by position — so the seeded observation stays "before the first measured write" however the
    // order is spelled. Their concatenation is `EXECUTION_ORDER` exactly when every apply channel
    // precedes every write-issuing one, which `ScalePointEvidence::sealed` re-checks positionally
    // and would reject outright if it ever stopped being true.
    let (apply_channels, writing_channels): (Vec<MeasurementChannel>, Vec<MeasurementChannel>) =
        MeasurementChannel::EXECUTION_ORDER
            .into_iter()
            .partition(|channel| MutationSchedule::of(*channel).is_none());

    let session = MeasuredSession::Unsubscribed { client };
    let session = match measure_channels(session, attempt, &apply_channels, &rows, &mut channels) {
        Ok(session) => session,
        Err(failure) => {
            return measured_failure(
                failure.release,
                channels,
                failure.observed,
                failure.kind,
                failure.error,
            )
        }
    };

    let seeded = session.client().read_measured_target(target);
    let before = match ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        seeded,
    ) {
        Ok(before) => before,
        Err(error) => {
            return measured_failure(
                session.into_release(),
                channels,
                MeasuredSampleBoundary::BeforeFirst,
                FailureKind::Infrastructure(InfrastructurePhase::ObservationRetention),
                error,
            )
        }
    };

    let session = match measure_channels(session, attempt, &writing_channels, &rows, &mut channels)
    {
        Ok(session) => session,
        Err(failure) => {
            return measured_failure(
                failure.release,
                channels,
                failure.observed,
                failure.kind,
                failure.error,
            )
        }
    };

    // One read, used twice: the window closes over the state these rows describe, and the same rows
    // become the retained artifact. A second read could disagree with the first.
    let final_rows = session.client().read_measured_target(target);
    let delivery = match meter.close(session.retained_subscriptions()) {
        Ok(delivery) => delivery,
        Err(error) => {
            return measured_failure(
                session.into_release(),
                channels,
                MeasuredSampleBoundary::BeforeFirst,
                FailureKind::Infrastructure(InfrastructurePhase::Sample),
                error,
            )
        }
    };
    let after = match ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        final_rows,
    ) {
        Ok(after) => after,
        Err(error) => {
            return measured_failure(
                session.into_release(),
                channels,
                MeasuredSampleBoundary::BeforeFirst,
                FailureKind::Infrastructure(InfrastructurePhase::ObservationRetention),
                error,
            )
        }
    };

    (
        session.into_release(),
        Ok((
            MeasuredAttempt {
                channels,
                before,
                after,
            },
            delivery,
        )),
    )
}

/// Walk `channels` through [`measure_channel`], pushing each evidence onto `measured` so a failure
/// can return whatever had completed.
///
/// The session is threaded by value rather than borrowed, because the reconnect replaces both the
/// client and its subscription: a borrow could not express that, and an in-place swap would need
/// something to leave behind.
fn measure_channels(
    mut session: MeasuredSession,
    attempt: AttemptKey,
    channels: &[MeasurementChannel],
    rows: &Arc<SubscriberRowCounter>,
    measured: &mut Vec<ChannelEvidence>,
) -> std::result::Result<MeasuredSession, ChannelFailure> {
    for channel in channels {
        let (advanced, evidence) = measure_channel(session, attempt, *channel, rows)?;
        session = advanced;
        measured.push(evidence);
    }
    Ok(session)
}

/// Build one measured window's return: the release state the caller must honour, and the classified
/// failure carrying the channel prefix already completed.
fn measured_failure(
    release: MeasuredRelease,
    measured: Vec<ChannelEvidence>,
    observed: MeasuredSampleBoundary,
    kind: FailureKind,
    error: Error,
) -> (
    MeasuredRelease,
    std::result::Result<(MeasuredAttempt, SubscriberDeliveryEvidence), MeasuredFailure>,
) {
    let stage = failure_stage(&measured, observed);
    (
        release,
        Err(MeasuredFailure {
            kind,
            stage,
            measured,
            error,
        }),
    )
}

/// The stage a measured window's failure is at: before its first measured sample only when *neither*
/// witness says otherwise.
///
/// Two witnesses, either sufficient. A non-empty prefix proves a sample was taken, since the cold
/// subscription runs first and its apply observation *is* the attempt's first measured sample. The
/// converse fails: that same channel can apply and then have its reduction refuse the duration,
/// leaving an observation with an empty prefix. Reading the prefix alone would record that measured
/// attempt as retry-eligible and omit the post-attempt reading it requires.
///
/// Split from [`measured_failure`], which needs a live client, so the rule is provable.
fn failure_stage(measured: &[ChannelEvidence], observed: MeasuredSampleBoundary) -> FailureStage {
    if measured.is_empty() && observed == MeasuredSampleBoundary::BeforeFirst {
        FailureStage::AfterPublishBeforeFirstSample
    } else {
        FailureStage::AfterFirstSample
    }
}

/// Measure one channel at this attempt's scale point, reducing its own samples to its own statistic.
///
/// A total match over the four channels, each advancing the session it needs. The estimands and
/// intervals are the client's; this stage picks the primitive, threads ownership, and classifies.
/// The two write-issuing channels drive [`MutationSchedule::of`] through the owner-preserving update
/// path; each [`ChannelEvidence`] constructor performs its own reduction, so no statistic is computed
/// here.
///
/// A session shape the frozen [`MeasurementChannel::EXECUTION_ORDER`] cannot produce is an internal
/// contradiction, not a measured outcome, so it panics rather than being recorded as an attempt
/// failure — the bridge [`schedule_retry`] makes for the same kind of unencoded invariant.
fn measure_channel(
    session: MeasuredSession,
    attempt: AttemptKey,
    channel: MeasurementChannel,
    rows: &Arc<SubscriberRowCounter>,
) -> std::result::Result<(MeasuredSession, ChannelEvidence), ChannelFailure> {
    let target = MeasuredTarget::of(attempt.role());

    match channel {
        MeasurementChannel::ColdSubscriptionApplyTime => {
            let MeasuredSession::Unsubscribed { client } = session else {
                panic!(
                    "the frozen execution order runs the cold subscription first, so nothing has \
                     subscribed on this connection yet"
                )
            };
            match client.subscribe_measured_target(target) {
                Ok((subscription, applied)) => reduced(
                    MeasuredSession::Subscribed {
                        client,
                        subscription,
                    },
                    ChannelEvidence::cold_subscription(applied),
                ),
                Err(failure) => {
                    let (kind, error) = classified(failure, InfrastructurePhase::Subscription);
                    Err(ChannelFailure {
                        release: MeasuredRelease::Connected(client),
                        observed: MeasuredSampleBoundary::BeforeFirst,
                        kind,
                        error,
                    })
                }
            }
        }

        MeasurementChannel::ReconnectApplyTime => {
            let MeasuredSession::Subscribed {
                client,
                subscription,
            } = session
            else {
                panic!(
                    "the frozen execution order runs the reconnect after the cold subscription, so \
                     this connection already has exactly one retained subscription"
                )
            };
            // Dropped with the connection it belongs to. The pinned SDK does not mark an applied
            // subscription inactive when its connection ends, so carrying this handle across the
            // reconnect would report a subscription that no longer exists as still delivering.
            drop(subscription);
            let previous_identity = client.measured_identity();

            match client.reconnect_preserving_token(target, rows) {
                Ok((client, subscription, applied)) => {
                    let reconnected_identity = client.measured_identity();
                    if reconnected_identity == previous_identity {
                        reduced(
                            MeasuredSession::Subscribed {
                                client,
                                subscription,
                            },
                            ChannelEvidence::reconnect(applied),
                        )
                    } else {
                        // Caught here rather than left to composition validation, which would
                        // otherwise see the Arm's read set empty out and classify a token-handling
                        // fault as the candidate's own security answer.
                        Err(ChannelFailure {
                            release: MeasuredRelease::Connected(client),
                            // The apply duration was observed, but a reconnect of a *different*
                            // identity is not an observation of this attempt's estimand, so it is
                            // not this attempt's first measured sample.
                            observed: MeasuredSampleBoundary::BeforeFirst,
                            kind: FailureKind::Infrastructure(InfrastructurePhase::Connect),
                            error: anyhow!(
                                "the reconnect was issued identity {} rather than the measured \
                                 identity {} its retained token names, so it did not measure this \
                                 subscriber reconnecting",
                                reconnected_identity.to_hex(),
                                previous_identity.to_hex(),
                            ),
                        })
                    }
                }
                // The teardown that precedes the reconnect failed, so the reconnect never began.
                // The release error travels as the release state; the attempt's own cause is stated
                // separately rather than duplicating one error into two roles.
                Err(ReconnectFailure::ReleaseFailed(error)) => Err(ChannelFailure {
                    release: MeasuredRelease::ReleasedWithError(error),
                    observed: MeasuredSampleBoundary::BeforeFirst,
                    kind: FailureKind::Infrastructure(InfrastructurePhase::Connect),
                    error: anyhow!(
                        "releasing the measured subscriber ahead of its token-preserving reconnect \
                         ended abnormally, so the reconnect never began; that release failure is \
                         reported separately as this attempt's resource-release diagnostic"
                    ),
                }),
                Err(ReconnectFailure::NotConnected(failure)) => {
                    let (kind, error) = classified(failure, InfrastructurePhase::Connect);
                    Err(ChannelFailure {
                        release: MeasuredRelease::Released,
                        observed: MeasuredSampleBoundary::BeforeFirst,
                        kind,
                        error,
                    })
                }
                Err(ReconnectFailure::NotResubscribed { client, failure }) => {
                    let (kind, error) = classified(failure, InfrastructurePhase::Subscription);
                    Err(ChannelFailure {
                        release: MeasuredRelease::Connected(client),
                        observed: MeasuredSampleBoundary::BeforeFirst,
                        kind,
                        error,
                    })
                }
            }
        }

        MeasurementChannel::PacedVisibleApplyLatency => {
            let MeasuredSession::Subscribed {
                client,
                subscription,
            } = session
            else {
                panic!(
                    "the frozen execution order runs the paced channel after the reconnect, so \
                     this connection already has exactly one retained subscription"
                )
            };
            match client.measure_paced_visible_batch(target, write_schedule(channel)) {
                Ok(samples) => reduced(
                    MeasuredSession::Subscribed {
                        client,
                        subscription,
                    },
                    ChannelEvidence::paced(samples),
                ),
                Err(failure) => {
                    let (kind, error) = classified(failure, InfrastructurePhase::Sample);
                    Err(ChannelFailure {
                        release: MeasuredRelease::Connected(client),
                        observed: MeasuredSampleBoundary::BeforeFirst,
                        kind,
                        error,
                    })
                }
            }
        }

        MeasurementChannel::SaturatedQueueGrowthPerWrite => {
            let MeasuredSession::Subscribed {
                client,
                subscription,
            } = session
            else {
                panic!(
                    "the frozen execution order runs the saturated channel last, so this \
                     connection already has exactly one retained subscription"
                )
            };
            match client.measure_saturated_batch(write_schedule(channel)) {
                Ok(timings) => reduced(
                    MeasuredSession::Subscribed {
                        client,
                        subscription,
                    },
                    ChannelEvidence::saturated(timings),
                ),
                Err(failure) => {
                    let (kind, error) = classified(failure, InfrastructurePhase::Sample);
                    Err(ChannelFailure {
                        release: MeasuredRelease::Connected(client),
                        observed: MeasuredSampleBoundary::BeforeFirst,
                        kind,
                        error,
                    })
                }
            }
        }
    }
}

/// This channel's measured-write schedule, for the two channels that have one.
///
/// Loud rather than fallible: [`MutationSchedule::of`] returns `None` for exactly the two apply
/// channels, and neither reaches this — asking for it there is a contradiction in the caller, not a
/// measured outcome.
fn write_schedule(channel: MeasurementChannel) -> MutationSchedule {
    MutationSchedule::of(channel)
        .expect("only the two write-issuing channels ask for a measured-write schedule")
}

/// Pair a completed channel with the session that produced it, or classify its reduction's refusal.
///
/// A refused statistic is a measured outcome, never an operational fault, so it is
/// [`NonpositiveStatistic`](FailureKind::NonpositiveStatistic) wherever it arises; the client is
/// still connected, since the reduction runs after the channel has finished with it.
///
/// This is the one place a failure is [`AfterFirst`](MeasuredSampleBoundary::AfterFirst) with an
/// empty prefix, and the whole reason the boundary is reported rather than inferred: reaching here
/// means the observation was taken and the refusal came after it.
fn reduced(
    session: MeasuredSession,
    evidence: Result<ChannelEvidence>,
) -> std::result::Result<(MeasuredSession, ChannelEvidence), ChannelFailure> {
    match evidence {
        Ok(evidence) => Ok((session, evidence)),
        Err(error) => Err(ChannelFailure {
            release: session.into_release(),
            observed: MeasuredSampleBoundary::AfterFirst,
            kind: FailureKind::NonpositiveStatistic,
            error,
        }),
    }
}

/// Translate one measured step's classified failure into this campaign's cause vocabulary.
///
/// The client knows *what* went wrong; the driver knows *where*, since one primitive serves more
/// than one channel. `phase` is that half, and only the infrastructure class carries it. Total over
/// the three causes, so a timeout cannot be re-labelled as infrastructure to make it retry-eligible.
fn classified(failure: MeasuredStepFailure, phase: InfrastructurePhase) -> (FailureKind, Error) {
    match failure {
        MeasuredStepFailure::Infrastructure(error) => (FailureKind::Infrastructure(phase), error),
        MeasuredStepFailure::Application(error) => (FailureKind::Application, error),
        MeasuredStepFailure::Timeout(error) => (FailureKind::Timeout, error),
    }
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
