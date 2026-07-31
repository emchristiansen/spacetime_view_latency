//! Derisking probe for the `EntityOwnerSenderView` candidate (spec c33f2e51, Revision Discussion):
//! does a sender-scoped view's cost track the subscriber's *own* visible cardinality, with the
//! backing table held constant?
//!
//! The seed-7 Pilot answered the complementary question — unrelated rows grow, own slice pinned at
//! ten — and found the candidate flat. Here the table is fixed at [`TOTAL_BACKING_ROWS`] and only
//! the partition between the measured identity and a non-connecting other owner moves, so
//! own-result growth is separated from backing growth by construction.
//!
//! Not a campaign stage: it adds no axis, moves no preregistered constant, and writes no campaign
//! ledger. It reuses the campaign's measured-write schedule and target/channel types unchanged, and
//! mirrors [`crate::entity_owner_smoke`]'s provisioning and teardown discipline.

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use spacetimedb_sdk::Identity;

use crate::client::connected_client::ConnectedClient;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::bindings::EntityOwner;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::observation::output_path::OutputPath;
use crate::params::EXPERIMENT_ISSUER;
use crate::plan::run_role::RunRole;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;
use crate::view_read_set_campaign::campaign_params::{
    GLOBAL_KEY_BASE, OWNED_KEY_BASE, SUBSCRIBER_VISIBLE_ROWS_BASELINE,
};
use crate::view_read_set_campaign::measured_target::MeasuredTarget;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;

/// Rows every attempt's table holds. Ten thousand joins Experiment B exactly, so the top rung —
/// where the measured identity owns the whole table — is comparable to a measurement that exists.
const TOTAL_BACKING_ROWS: u64 = 10_000;

/// How many of the fixed population the measured identity owns, per rung.
const VISIBLE_ROWS_LADDER: [u64; 5] = [10, 100, 1_000, 4_000, 10_000];

const PROBE_BLOCKS: u32 = 2;
const OTHER_OWNER_SUBJECT: &str = "entity-owner-visible-rows-probe-other-owner";
const SEED_PAYLOAD: &str = "entity-owner-visible-rows-probe-seed-payload";
const ATTEMPT_ORDER_DOMAIN: &str = "entity-owner-visible-rows-probe:attempt-order";

/// The gate every attempt waits behind, fixed rather than exposed as flags: Experiment B's arms ran
/// behind exactly these thresholds, so an attempt admitted by a different gate would not be
/// comparable to them. The waiter checks its deadline once per sample, so the longest possible wait
/// is eight minutes plus one fifteen-second interval.
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

/// Tells the waiter to exit with a distinct status when it reaches its deadline and refuses, so a
/// refusal is distinguishable from the raise every operational failure produces. Three because zero
/// is admission, one is what Nushell raises, and two is argument misuse.
const REFUSAL_EXIT_CODE_FLAG: &str = "--refusal-exit-code";
const GATE_REFUSAL_EXIT_CODE: i32 = 3;

/// The ladder must ascend, must fit the table, must not collide with the unrelated key space, and
/// must own the whole measured write slice — the schedule cycles the first
/// `SUBSCRIBER_VISIBLE_ROWS_BASELINE` owned keys, so a rung below it would write a row the measured
/// identity does not hold.
const _: () = {
    let mut rung = 1usize;
    while rung < VISIBLE_ROWS_LADDER.len() {
        assert!(VISIBLE_ROWS_LADDER[rung - 1] < VISIBLE_ROWS_LADDER[rung]);
        rung += 1;
    }
    assert!(VISIBLE_ROWS_LADDER[VISIBLE_ROWS_LADDER.len() - 1] <= TOTAL_BACKING_ROWS);
    assert!(VISIBLE_ROWS_LADDER[0] >= SUBSCRIBER_VISIBLE_ROWS_BASELINE);
    assert!(OWNED_KEY_BASE + TOTAL_BACKING_ROWS <= GLOBAL_KEY_BASE);
};

/// What one fresh server measures.
///
/// The Control carries no rung because its subscription is the whole table, whose cardinality is
/// `TOTAL_BACKING_ROWS` at every rung — so a Control naming a visible-rows value would name a number
/// that does not describe it, and one per rung would re-measure the same configuration five times.
/// [`Self::block`] is the only constructor, so a block holds every rung once and exactly one
/// Control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeAttempt {
    Arm { visible_rows: u64 },
    Control,
}

impl ProbeAttempt {
    fn block() -> [Self; VISIBLE_ROWS_LADDER.len() + 1] {
        let [a, b, c, d, e] = VISIBLE_ROWS_LADDER.map(|visible_rows| Self::Arm { visible_rows });
        [a, b, c, d, e, Self::Control]
    }

    fn role(self) -> RunRole {
        match self {
            Self::Arm { .. } => RunRole::Arm,
            Self::Control => RunRole::Control,
        }
    }

    fn measured_target(self) -> MeasuredTarget {
        MeasuredTarget::of(self.role())
    }

    /// The Control seeds the Pilot's baseline slice: its subscription ignores the partition, but its
    /// measured writes still target owned keys, so the write path matches the Arm's.
    fn owned_rows(self) -> u64 {
        match self {
            Self::Arm { visible_rows } => visible_rows,
            Self::Control => SUBSCRIBER_VISIBLE_ROWS_BASELINE,
        }
    }

    fn expected_subscribed_rows(self) -> u64 {
        match self {
            Self::Arm { visible_rows } => visible_rows,
            Self::Control => TOTAL_BACKING_ROWS,
        }
    }

    fn other_owner_rows(self) -> u64 {
        TOTAL_BACKING_ROWS - self.owned_rows()
    }

    fn canonical_tag(self) -> String {
        match self {
            Self::Arm { visible_rows } => format!("arm-visible-{visible_rows}"),
            Self::Control => "control".to_string(),
        }
    }
}

/// One channel's samples, summarised and retained raw.
#[derive(Serialize)]
struct ChannelSummary {
    samples: usize,
    mean_nanos: u128,
    p50_nanos: u128,
    p99_nanos: u128,
    min_nanos: u128,
    max_nanos: u128,
    raw_nanos: Vec<u128>,
}

impl ChannelSummary {
    fn of(mut nanos: Vec<u128>) -> Result<Self> {
        ensure!(
            !nanos.is_empty(),
            "a channel must yield at least one sample"
        );
        let raw_nanos = nanos.clone();
        nanos.sort_unstable();
        let samples = nanos.len();
        let total: u128 = nanos.iter().sum();
        let p99_index = ((samples as f64) * 0.99) as usize;
        Ok(Self {
            samples,
            mean_nanos: total / samples as u128,
            p50_nanos: nanos[samples / 2],
            p99_nanos: nanos[p99_index.min(samples - 1)],
            min_nanos: nanos[0],
            max_nanos: nanos[samples - 1],
            raw_nanos,
        })
    }
}

/// One attempt's evidence line.
#[derive(Serialize)]
struct ProbeRecord {
    block: u32,
    attempt: String,
    role: &'static str,
    owned_rows: u64,
    other_owner_rows: u64,
    total_backing_rows: u64,
    expected_subscribed_rows: u64,
    observed_subscribed_rows: usize,
    cold_subscription_nanos: u128,
    paced: ChannelSummary,
    saturated: ChannelSummary,
}

/// Run every block's attempts in seed-derived order, each behind the host gate and on its own fresh
/// server, appending each attempt's evidence as it completes.
pub(crate) fn entity_owner_visible_rows_probe(
    listen: ListenAddress,
    module_wasm: &Path,
    host_waiter: &Path,
    output: &OutputPath,
    seed: ScheduleSeed,
) -> Result<()> {
    // Both resolved before anything is created, so a mistyped waiter or an occupied ledger path
    // fails immediately instead of after an eight-minute wait.
    let host_waiter = resolve_host_waiter(host_waiter)?;
    ensure!(
        !output.path().exists(),
        "the probe ledger {} already exists; evidence is never overwritten",
        output.path().display()
    );

    let plan: Vec<(u32, ProbeAttempt)> = (0..PROBE_BLOCKS)
        .flat_map(|block| {
            ordered_attempts(block, seed)
                .into_iter()
                .map(move |attempt| (block, attempt))
        })
        .collect();

    // Opened on the first attempt admitted through the gate, never before it: a run refused at the
    // gate has measured nothing, so it must leave the requested path untouched and reusable.
    let mut ledger: Option<File> = None;
    for (block, attempt) in plan {
        println!("probe: block {block} attempt {}", attempt.canonical_tag());
        if !wait_for_free_ish_host(&host_waiter, block, attempt)? {
            println!(
                "probe: block {block} attempt {} skipped, the host gate refused",
                attempt.canonical_tag()
            );
            continue;
        }
        let ledger = match &mut ledger {
            Some(ledger) => ledger,
            empty => empty.insert(create_ledger(output)?),
        };

        let record = run_attempt(listen, module_wasm, block, attempt)?;
        let line = serde_json::to_string(&record).context("encoding a probe record")?;
        writeln!(ledger, "{line}").context("appending a probe record")?;
        // Flushed per attempt: a probe killed partway keeps the attempts it finished, and the next
        // attempt's gate wait begins with this one already durable.
        ledger.flush().context("flushing a probe record")?;
    }
    Ok(())
}

/// Create the ledger exclusively, as the Pilot's is: this is the authority on the path being free,
/// the earlier check only being what makes an occupied path fail fast.
fn create_ledger(output: &OutputPath) -> Result<File> {
    File::options()
        .write(true)
        .create_new(true)
        .open(output.path())
        .with_context(|| format!("creating the probe ledger {}", output.path().display()))
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

/// Block until the host is free-ish. Runs immediately before an attempt provisions anything, so a
/// contended host is never measured: no server is started and no evidence line is written for this
/// attempt. Inherits stdout, so its per-sample diagnostics stream during a wait that can last
/// minutes.
///
/// Returns `false` when the waiter reached its deadline and refused, which the caller skips over so
/// the remaining rungs stay runnable. Any other nonzero exit is still a hard error: those are the
/// waiter's operational raises, and reading one as a refusal would skip every attempt in seconds
/// while looking exactly like a busy host.
fn wait_for_free_ish_host(host_waiter: &Path, block: u32, attempt: ProbeAttempt) -> Result<bool> {
    let status = Command::new(host_waiter)
        .args(HOST_WAITER_ARGS)
        .arg(REFUSAL_EXIT_CODE_FLAG)
        .arg(GATE_REFUSAL_EXIT_CODE.to_string())
        .status()
        .with_context(|| format!("running the host waiter {}", host_waiter.display()))?;
    if status.code() == Some(GATE_REFUSAL_EXIT_CODE) {
        return Ok(false);
    }
    ensure!(
        status.success(),
        "the host waiter {} exited unsuccessfully ({status}), so block {block} attempt {} was not \
         started",
        host_waiter.display(),
        attempt.canonical_tag(),
    );
    Ok(true)
}

/// One block's attempts, permuted by a domain-separated digest of `(seed, block, attempt)`. Sorting
/// is a permutation, so the result is the block reordered — nothing added, dropped or duplicated.
fn ordered_attempts(block: u32, seed: ScheduleSeed) -> Vec<ProbeAttempt> {
    let mut keyed: Vec<([u8; 32], String, ProbeAttempt)> = ProbeAttempt::block()
        .into_iter()
        .map(|attempt| {
            let tag = attempt.canonical_tag();
            let subject = format!(
                "{ATTEMPT_ORDER_DOMAIN};seed={};block={block};attempt={tag}",
                seed.get()
            );
            let mut hasher = Sha256::new();
            hasher.update(subject.as_bytes());
            (hasher.finalize().into(), tag, attempt)
        })
        .collect();
    keyed.sort_by(|(left, left_tag, _), (right, right_tag, _)| {
        left.cmp(right).then_with(|| left_tag.cmp(right_tag))
    });
    keyed.into_iter().map(|(_, _, attempt)| attempt).collect()
}

/// Provision, publish, measure, and tear down one attempt — the smoke driver's acquisition pipeline.
fn run_attempt(
    listen: ListenAddress,
    module_wasm: &Path,
    block: u32,
    attempt: ProbeAttempt,
) -> Result<ProbeRecord> {
    let distribution = VerifiedDistribution::resolve()?;
    let staged = StagedModuleWasm::load(module_wasm, WasmSha256::new(MODULE_WASM_SHA256))?;

    let server = match RunningPinnedServer::start(&distribution, listen) {
        Ok(server) => server,
        Err(error) => {
            let mut errors = vec![error];
            if let Err(e) = staged.cleanup() {
                errors.push(e);
            }
            return Err(into_error(errors));
        }
    };

    let artifact = match server.publish(&distribution, &staged) {
        Ok(artifact) => artifact,
        Err(error) => {
            let mut errors = vec![error];
            if let Err(e) = server.shutdown() {
                errors.push(e);
            }
            if let Err(e) = staged.cleanup() {
                errors.push(e);
            }
            return Err(into_error(errors));
        }
    };

    let database_identity = artifact.database_identity().canonical_hex();
    let resources = RunResources::new(server, staged);
    let outcome = drive(resources.server(), &database_identity, block, attempt);

    let mut errors = Vec::new();
    let record = match outcome {
        Ok(record) => Some(record),
        Err(e) => {
            errors.push(e);
            None
        }
    };
    if let Err(e) = resources.teardown() {
        errors.push(e);
    }
    match record {
        Some(record) if errors.is_empty() => Ok(record),
        _ => Err(into_error(errors)),
    }
}

/// Connect, measure, then disconnect unconditionally — the smoke driver's `drive`, returning
/// evidence instead of unit.
fn drive(
    server: &RunningPinnedServer,
    database_identity: &str,
    block: u32,
    attempt: ProbeAttempt,
) -> Result<ProbeRecord> {
    let server_url = server.listen().client_url();
    let client = ConnectedClient::connect(&server_url, database_identity)
        .context("connecting the measured subscriber")?;

    let outcome = measure(&client, block, attempt);
    let disconnect = client
        .disconnect()
        .context("disconnecting the measured subscriber");

    match (outcome, disconnect) {
        (Ok(record), Ok(())) => Ok(record),
        (Err(body), Ok(())) => Err(body),
        (Ok(_), Err(teardown)) => Err(teardown),
        (Err(body), Err(teardown)) => Err(anyhow!(
            "the probe attempt failed and disconnecting the client afterward also failed:\n  \
             [1] {body:#}\n  [2] {teardown:#}"
        )),
    }
}

/// Seed the partition, check it, subscribe, run E2 paced then E1 saturated, and check the after
/// state. The channel order is the campaign's: saturated runs last, where its queue pressure cannot
/// perturb an unmeasured channel.
fn measure(client: &ConnectedClient, block: u32, attempt: ProbeAttempt) -> Result<ProbeRecord> {
    let owner = client.measured_identity();
    let other_owner = Identity::from_claims(EXPERIMENT_ISSUER, OTHER_OWNER_SUBJECT);

    for offset in 0..attempt.owned_rows() {
        client.insert_entity_owner(OWNED_KEY_BASE + offset, owner, SEED_PAYLOAD.to_string())?;
    }
    for offset in 0..attempt.other_owner_rows() {
        client.insert_entity_owner(
            GLOBAL_KEY_BASE + offset,
            other_owner,
            SEED_PAYLOAD.to_string(),
        )?;
    }

    let target = attempt.measured_target();
    let (_subscription, cold) = client
        .subscribe_measured_target(target)
        .map_err(|failure| failure.into_error())
        .context("subscribing the measured target")?;

    let subscribed = client.read_measured_target(target);
    ensure!(
        subscribed.len() as u64 == attempt.expected_subscribed_rows(),
        "{} subscribed to {} rows, expected exactly {}",
        attempt.canonical_tag(),
        subscribed.len(),
        attempt.expected_subscribed_rows(),
    );
    if matches!(attempt, ProbeAttempt::Arm { .. }) {
        ensure!(
            subscribed.iter().all(|row| row.owner == owner),
            "the sender-scoped view leaked a row the measured identity does not own",
        );
    }

    let paced_schedule = schedule_for(MeasurementChannel::PacedVisibleApplyLatency)?;
    let paced = client
        .measure_paced_visible_batch(target, paced_schedule)
        .map_err(|failure| failure.into_error())
        .context("the paced visible-apply channel")?;

    let saturated_schedule = schedule_for(MeasurementChannel::SaturatedQueueGrowthPerWrite)?;
    let saturated = client
        .measure_saturated_batch(saturated_schedule)
        .map_err(|failure| failure.into_error())
        .context("the saturated queue-growth channel")?;

    check_after_state(client, target, attempt, saturated_schedule)?;

    Ok(ProbeRecord {
        block,
        attempt: attempt.canonical_tag(),
        role: match attempt.role() {
            RunRole::Arm => "arm",
            RunRole::Control => "control",
        },
        owned_rows: attempt.owned_rows(),
        other_owner_rows: attempt.other_owner_rows(),
        total_backing_rows: TOTAL_BACKING_ROWS,
        expected_subscribed_rows: attempt.expected_subscribed_rows(),
        observed_subscribed_rows: subscribed.len(),
        cold_subscription_nanos: cold.nanos(),
        paced: ChannelSummary::of(paced.samples().iter().map(|s| s.nanos()).collect())?,
        saturated: ChannelSummary::of(
            saturated
                .writes()
                .iter()
                .map(|write| write.latency_nanos())
                .collect(),
        )?,
    })
}

fn schedule_for(channel: MeasurementChannel) -> Result<MutationSchedule> {
    MutationSchedule::of(channel)
        .ok_or_else(|| anyhow!("{channel:?} issues no measured writes, so the probe cannot use it"))
}

/// Every measured write replaces a payload in place, so afterwards cardinality must be unchanged and
/// each written key must hold the payload of the *last* write that targeted it.
fn check_after_state(
    client: &ConnectedClient,
    target: MeasuredTarget,
    attempt: ProbeAttempt,
    saturated: MutationSchedule,
) -> Result<()> {
    let mut expected: BTreeMap<u64, String> = BTreeMap::new();
    for write in saturated.writes() {
        expected.insert(saturated.target_key(write), saturated.payload(write));
    }

    let rows = client.read_measured_target(target);
    ensure!(
        rows.len() as u64 == attempt.expected_subscribed_rows(),
        "{} holds {} rows after its measured writes, expected the cardinality to be unchanged at {}",
        attempt.canonical_tag(),
        rows.len(),
        attempt.expected_subscribed_rows(),
    );

    let observed: BTreeMap<u64, &EntityOwner> =
        rows.iter().map(|row| (row.entity_uuid, row)).collect();
    for (entity_uuid, payload) in &expected {
        let Some(row) = observed.get(entity_uuid) else {
            bail!(
                "{} lost the measured key {entity_uuid} its writes targeted",
                attempt.canonical_tag()
            );
        };
        ensure!(
            &row.record == payload,
            "measured key {entity_uuid} holds {:?}, expected the last write's payload {payload:?}",
            row.record,
        );
    }
    Ok(())
}
