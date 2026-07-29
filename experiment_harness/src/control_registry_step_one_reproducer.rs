//! `ControlRegistry` Step 1 (spec c33f2e51): capability and semantics for site 4's discovery pair,
//! on one fresh isolated server. **Nothing here is timed**, and no attempt inventory, E2, or E3 run
//! is authorized by anything it proves.
//!
//! **What it proves.** That the pinned release can publish and materialize a reducer-maintained
//! O(K) current-state registry alongside the O(N) history it summarizes; that Arm A
//! (`control_registry_all_view`), Diagnostic Arm B (`control_activity_latest_by_control_view`), and
//! the base registry table agree exactly on the same K logical rows; that they still agree after
//! live repeat activity; and that all four fail-loud preconditions **roll back** rather than merely
//! refuse.
//!
//! **The writer contract, which the seeding shape is not free to violate.** Every row here is
//! written by exactly one of two reducers: [`record_first_control_activity`] once per control, then
//! [`record_control_activity`] for every subsequent event. There is **no bulk seed path**, and the
//! module's pre-existing history-only `insert_control_activity` is **forbidden** in this reproducer
//! — it writes an audit row without registry maintenance, so a single call would falsify the
//! equality this milestone exists to prove. That reducer stays untouched because it belongs to the
//! separate sender-view candidate, whose pending write estimand its cost must keep representing;
//! nothing in the schema prevents mixing the paths, and this driver's discipline plus its
//! per-phase composition checks are what keep the workload inside the contract.
//!
//! **The invariant.** Under that contract `registry.last_ts == max(ts)` over each control's history
//! holds by induction at every committed state: the base case commits the first audit row and its
//! registry row in one transaction, and every later write requires a strictly greater `ts` and
//! advances `last_ts` to it in one transaction. This is a property of *how each state was reached*,
//! not only of the final one, which is why the seeding order is forced rather than convenient.
//!
//! **What Arm B is and is not.** It is a `DiagnosticOnly` O(N) comparator reinstated for exactly
//! this equality proof and the later descriptive screen. Its scan reaches the table through
//! macro-generated in-crate plumbing and it may never support a production recommendation.
//!
//! Mirrors [`crate::control_activity_latest_by_control_view_reproducer`]'s provisioning and teardown
//! discipline, which in turn mirrors [`crate::entity_owner_smoke`]'s.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, ensure, Context, Result};
use spacetimedb_sdk::{Identity, Timestamp};

use crate::client::connected_client::ConnectedClient;
use crate::client::view_event_shape::ViewEventShape;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::bindings::{ControlActivity, ControlRegistry};
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;

/// Distinct controls — the spec's frozen `K`, and the governing subscriber-visible baseline.
const CONTROL_COUNT: u64 = 10;

/// Seeded history rows — the spec's frozen `N`.
const HISTORY_ROW_COUNT: u64 = 1_000;

/// History rows per control, derived so the two frozen constants cannot disagree with the seeding
/// loop about their own composition.
const ROWS_PER_CONTROL: u64 = HISTORY_ROW_COUNT / CONTROL_COUNT;

/// The derivation above is only faithful when the division is exact. Integer division would
/// otherwise truncate, and the seeding loop would write `K * ROWS_PER_CONTROL` rows while the
/// composition checks assert against `N` — the two constants disagreeing about the composition they
/// are supposed to fix. The spec freezes `N / K = 100` exactly, so this is a compile-time property
/// rather than something the driver should be able to get wrong at run time.
const _: () = assert!(
    HISTORY_ROW_COUNT % CONTROL_COUNT == 0,
    "HISTORY_ROW_COUNT must divide evenly by CONTROL_COUNT: the frozen composition is N/K rows per \
     control exactly",
);

/// Fixed literal `control_uuid` base. Not the object of study; fixed so seeded state is reproducible
/// from the seed alone.
const CONTROL_UUID_BASE: u64 = 9_000;

/// Fixed literal base timestamp, microseconds since the Unix epoch — deterministic rather than
/// clock-dependent, the discipline the empty-view reproducer was corrected to follow.
const TS_BASE_MICROS: i64 = 1_700_000_000_000_000;

/// Microseconds between consecutive activity timestamps. Every row's `ts` is one step past the
/// previous **global** row's, so all timestamps are globally unique and each control's history is
/// strictly increasing — which the repeat reducer requires and the latest-per-control comparator
/// needs to be unambiguous.
const TS_STEP_MICROS: i64 = 1_000;

/// Fixed identity byte pattern base. Each control gets its own identity so the equality checks
/// compare a column that actually varies across rows; a single shared identity would let an
/// identity-dropping bug pass.
const IDENTITY_BYTE_BASE: u8 = 0x40;

/// Wait budget for a phase's deliveries to reach the subscriber caches.
///
/// Matched to the harness's other single-round-trip budgets. Elapsing is a real failure of this
/// milestone — the caches did not converge — never a retryable infrastructure fault.
const CONVERGENCE_TIMEOUT: Duration = Duration::from_secs(60);

/// Pause between convergence polls. Short enough that it adds no meaningful latency to a milestone
/// that is never timed, long enough to remove the hot-spin starvation pressure on the delivery it
/// waits for. Scheduler behavior on an oversubscribed host is not something a sleep can guarantee.
const CONVERGENCE_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// A control's full logical current-state row, independent of which relation produced it.
///
/// Arm A and the base registry yield [`ControlRegistry`]; Arm B yields [`ControlActivity`], which
/// additionally carries the audit row's `id`. Projecting both to this shape is what lets the three
/// be compared as *the same* logical rows rather than by coincidence of type — and it is exactly the
/// projection the spec fixes, `(control_uuid, user_identity, ts)`, with `control_uuid` as the key.
type CurrentState = BTreeMap<u64, (Identity, Timestamp)>;

/// Provision, publish, seed, subscribe, and verify Step 1 end to end, tearing the server down on
/// every exit path.
pub(crate) fn control_registry_step_one_reproducer(
    listen: ListenAddress,
    module_wasm: &Path,
) -> Result<()> {
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

    let outcome = drive(resources.server(), &database_identity);

    let mut errors = Vec::new();
    if let Err(e) = outcome {
        errors.push(e);
    }
    if let Err(e) = resources.teardown() {
        errors.push(e);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(into_error(errors))
    }
}

/// Connect the subscriber, run the milestone, then disconnect unconditionally.
fn drive(server: &RunningPinnedServer, database_identity: &str) -> Result<()> {
    let server_url = server.listen().client_url();
    let client = ConnectedClient::connect(&server_url, database_identity)
        .context("connecting the registry subscriber")?;

    let outcome = run_step_one(&client);
    let disconnect = client
        .disconnect()
        .context("disconnecting the registry subscriber");

    match (outcome, disconnect) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(body), Ok(())) => Err(body),
        (Ok(()), Err(teardown)) => Err(teardown),
        (Err(body), Err(teardown)) => Err(anyhow!(
            "the ControlRegistry Step 1 reproducer failed and disconnecting the client afterward \
             also failed:\n  [1] {body:#}\n  [2] {teardown:#}"
        )),
    }
}

/// Seed through the two atomic reducers, prove three-way equality, prove it survives live repeat
/// activity, then prove all four refusals roll back.
fn run_step_one(client: &ConnectedClient) -> Result<()> {
    seed(client)?;

    let (events_tx, events_rx) = mpsc::channel::<ViewEventShape>();
    client.observe_control_registry_all_view(events_tx);

    client
        .subscribe_control_registry_all_view()
        .context("subscribing to Arm A control_registry_all_view")?;
    client
        .subscribe_control_activity_latest_by_control_view()
        .context("subscribing to Arm B control_activity_latest_by_control_view")?;
    client
        .subscribe_control_registry()
        .context("subscribing to base control_registry")?;
    client
        .subscribe_control_activity()
        .context("subscribing to base control_activity")?;

    let expected = expected_after_seeding();
    let seeded_history = expected_history(false);
    check_composition(client, &expected, &seeded_history, "after seeding")?;
    let snapshot_events = collect_events(&events_rx, CONTROL_COUNT, "the Arm A initial snapshot")?;

    // Live repeat activity: one strictly later event per control, through the repeat reducer.
    let expected_after_repeat = expected_after_repeat_round();
    let repeat_history = expected_history(true);
    for control_index in 0..CONTROL_COUNT {
        let id = repeat_round_id(control_index);
        client
            .record_control_activity(id, control_uuid(control_index), activity_ts(id))
            .with_context(|| format!("repeat activity for control index {control_index}"))?;
    }
    // Every repeat call above has returned, and the pinned SDK invokes a successful reducer's row
    // callbacks *before* its completion callback (`db_connection.rs:216-219`, whose `apply_update`
    // runs `invoke_row_callbacks` at `:312`). So the whole repeat round's Arm A deliveries are
    // already queued here, and the drain inside `collect_events` is exhaustive rather than a
    // prefix — which is what makes the recorded shape a recording instead of an assumption.
    let repeat_events = collect_events(&events_rx, CONTROL_COUNT, "the Arm A repeat round")?;
    await_convergence(client, &expected_after_repeat, &repeat_history)?;
    // This is also where identity preservation is proven, and the only place it can be: the
    // expectation carries each control's `control_identity`, `CurrentState` compares identity as
    // part of the value, and the repeat reducer was never given one to pass. Comparing the two
    // locally-built expectations to each other would prove nothing, since both derive identity from
    // the same function.
    check_composition(
        client,
        &expected_after_repeat,
        &repeat_history,
        "after repeat activity",
    )?;
    let refusals = prove_refusals_roll_back(client, &expected_after_repeat, &repeat_history)?;

    println!(
        "ControlRegistry Step 1: seeded N={HISTORY_ROW_COUNT} history rows across K={CONTROL_COUNT} \
         controls ({ROWS_PER_CONTROL} each) through one first-activity call then \
         {} repeat calls per control — the history-only reducer was never used. Arm A \
         (control_registry_all_view), Diagnostic Arm B (control_activity_latest_by_control_view), \
         and base control_registry agree on exactly {CONTROL_COUNT} full logical rows; base \
         control_activity holds {HISTORY_ROW_COUNT} rows over exactly {CONTROL_COUNT} controls. \
         After one strictly later activity per control all three reconverge on {CONTROL_COUNT} rows \
         with {} history rows, every identity preserved and every last_ts advanced. The whole audit \
         history was compared row for row against an independently constructed expectation at every \
         phase, so per-control multiplicity, identity, id, and timestamp are proven rather than \
         inferred from totals. Arm A delivered [{}] on the initial snapshot and [{}] on the repeat \
         round, each drained to exhaustion rather than truncated at K. All four refusals failed \
         loud and rolled back with both tables logically identical across every column:\n{}",
        ROWS_PER_CONTROL - 1,
        HISTORY_ROW_COUNT + CONTROL_COUNT,
        describe(&snapshot_events),
        describe(&repeat_events),
        refusals
            .iter()
            .map(|line| format!("  - {line}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    Ok(())
}

/// Seed the frozen composition: one first-activity call per control, then `ROWS_PER_CONTROL - 1`
/// repeat calls each.
///
/// The order is forced by the reducers' own preconditions rather than chosen: a repeat call before
/// its control's first call is refused, and a second first call is refused. That is what makes the
/// induction an argument about every committed state rather than about the final one.
fn seed(client: &ConnectedClient) -> Result<()> {
    for control_index in 0..CONTROL_COUNT {
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

    for occurrence in 1..ROWS_PER_CONTROL {
        for control_index in 0..CONTROL_COUNT {
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

/// A distinct fixed identity per control, so the compared rows carry a column that actually varies.
fn control_identity(control_index: u64) -> Identity {
    let mut bytes = [0u8; 32];
    bytes[31] = IDENTITY_BYTE_BASE + u8::try_from(control_index).expect("K fits u8");
    Identity::from_byte_array(bytes)
}

/// The activity id for a control's `occurrence`-th seeded row. Round-robin across controls, so ids
/// are globally unique and each control's ids increase with its occurrence.
fn activity_id(control_index: u64, occurrence: u64) -> u64 {
    occurrence * CONTROL_COUNT + control_index
}

/// The activity id for the repeat round's event for one control — continuing the seed's id space
/// past every seeded row so no id can collide.
fn repeat_round_id(control_index: u64) -> u64 {
    HISTORY_ROW_COUNT + control_index
}

/// The timestamp for activity `id` — strictly increasing in `id` across the whole run, so every
/// timestamp is globally unique and each control's history is strictly increasing.
fn activity_ts(id: u64) -> Timestamp {
    let offset = i64::try_from(id).expect("the activity id space fits i64");
    Timestamp::from_micros_since_unix_epoch(TS_BASE_MICROS + offset * TS_STEP_MICROS)
}

/// The current state expected after seeding: each control's latest seeded row.
fn expected_after_seeding() -> CurrentState {
    (0..CONTROL_COUNT)
        .map(|control_index| {
            let id = activity_id(control_index, ROWS_PER_CONTROL - 1);
            (
                control_uuid(control_index),
                (control_identity(control_index), activity_ts(id)),
            )
        })
        .collect()
}

/// The current state expected after the repeat round.
fn expected_after_repeat_round() -> CurrentState {
    (0..CONTROL_COUNT)
        .map(|control_index| {
            let id = repeat_round_id(control_index);
            (
                control_uuid(control_index),
                (control_identity(control_index), activity_ts(id)),
            )
        })
        .collect()
}

/// The complete audit history at a phase, in the canonical order [`sorted_activity`] produces.
///
/// Comparing against this is what makes the history proof exact. Totals cannot do it: a row written
/// under the wrong `control_uuid`, or carrying the wrong identity or timestamp, leaves both the row
/// count and the distinct-control count untouched while silently changing the per-control
/// composition the spec freezes.
type History = Vec<(u64, u64, Identity, Timestamp)>;

/// Every history row the writer contract must have produced, built independently of anything the
/// server returned.
///
/// `include_repeat_round` adds the one strictly later row per control that the live phase appends.
fn expected_history(include_repeat_round: bool) -> History {
    let mut rows: History = Vec::new();
    for control_index in 0..CONTROL_COUNT {
        for occurrence in 0..ROWS_PER_CONTROL {
            let id = activity_id(control_index, occurrence);
            rows.push((
                id,
                control_uuid(control_index),
                control_identity(control_index),
                activity_ts(id),
            ));
        }
        if include_repeat_round {
            let id = repeat_round_id(control_index);
            rows.push((
                id,
                control_uuid(control_index),
                control_identity(control_index),
                activity_ts(id),
            ));
        }
    }
    rows.sort();
    rows
}

/// Project [`ControlRegistry`] rows to the comparable current state.
fn registry_state(rows: &[ControlRegistry]) -> CurrentState {
    rows.iter()
        .map(|row| (row.control_uuid, (row.user_identity, row.last_ts)))
        .collect()
}

/// Project Arm B's [`ControlActivity`] rows to the comparable current state, dropping the audit
/// row's `id` — the projection the spec fixes, and the only difference between the two arms' shapes.
fn activity_state(rows: &[ControlActivity]) -> CurrentState {
    rows.iter()
        .map(|row| (row.control_uuid, (row.user_identity, row.ts)))
        .collect()
}

/// Prove Arm A, Arm B, and the base registry all equal `expected`, and that base `control_activity`
/// holds exactly `history_rows` rows over exactly K controls.
///
/// Cardinality is asserted on the raw row vectors **before** any projection, because projecting into
/// a map keyed by `control_uuid` silently collapses a duplicated key — which is precisely the
/// violation a one-row-per-control relation must not be allowed to hide.
fn check_composition(
    client: &ConnectedClient,
    expected: &CurrentState,
    expected_history: &History,
    phase: &str,
) -> Result<()> {
    let arm_a = client.read_control_registry_all_view();
    let arm_b = client.read_control_activity_latest_by_control_view();
    let base_registry = client.read_control_registry();
    let base_activity = client.read_control_activity();

    for (label, len) in [
        ("Arm A control_registry_all_view", arm_a.len()),
        ("Arm B control_activity_latest_by_control_view", arm_b.len()),
        ("base control_registry", base_registry.len()),
    ] {
        ensure!(
            len == CONTROL_COUNT as usize,
            "{phase}: {label} returned {len} rows, expected exactly K={CONTROL_COUNT}",
        );
    }

    for (label, state) in [
        ("Arm A control_registry_all_view", registry_state(&arm_a)),
        (
            "Arm B control_activity_latest_by_control_view",
            activity_state(&arm_b),
        ),
        ("base control_registry", registry_state(&base_registry)),
    ] {
        ensure!(
            &state == expected,
            "{phase}: {label} projected to {state:?}, expected {expected:?}",
        );
    }

    // Totals first, because they name the failure most directly when they are what broke.
    ensure!(
        base_activity.len() == expected_history.len(),
        "{phase}: base control_activity holds {} rows, expected {}",
        base_activity.len(),
        expected_history.len(),
    );
    let distinct: BTreeSet<u64> = base_activity.iter().map(|row| row.control_uuid).collect();
    ensure!(
        distinct.len() == CONTROL_COUNT as usize,
        "{phase}: base control_activity spans {} distinct control_uuids, expected exactly \
         K={CONTROL_COUNT}",
        distinct.len(),
    );

    // Then the whole history, row for row. This is the check that actually pins the frozen
    // composition: per-control multiplicity, every identity, every id, and every timestamp —
    // including the non-latest rows, which no current-state projection can see.
    let observed_history = sorted_activity(&base_activity);
    if &observed_history != expected_history {
        let (index, observed_row, expected_row) = first_history_divergence(
            &observed_history,
            expected_history,
        );
        return Err(anyhow!(
            "{phase}: base control_activity does not match the expected history. First divergence \
             at canonical index {index}: observed {observed_row}, expected {expected_row}",
        ));
    }

    Ok(())
}

/// Locate the first differing canonical position between an observed and expected history, so a
/// mismatch reports the offending row rather than two thousand-row dumps.
fn first_history_divergence(
    observed: &History,
    expected: &History,
) -> (usize, String, String) {
    let render = |row: Option<&(u64, u64, Identity, Timestamp)>| match row {
        Some((id, control_uuid, identity, ts)) => {
            format!("(id={id}, control_uuid={control_uuid}, identity={identity}, ts={ts:?})")
        }
        None => "<no row>".to_string(),
    };
    let index = observed
        .iter()
        .zip(expected.iter())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| observed.len().min(expected.len()));
    (index, render(observed.get(index)), render(expected.get(index)))
}

/// Block until all four relations reflect the post-repeat state, so the composition check that
/// follows is not racing delivery.
///
/// The repeat reducers' completion callbacks confirm the writes committed, not that every
/// subscription has applied them. Re-reading under one absolute deadline until the caches agree is
/// what separates "not yet delivered" from "wrong", and a timeout here means the caches never
/// converged — a real failure of this milestone.
///
/// The poll backs off between passes rather than spinning. Each pass re-collects all four caches,
/// including the whole `N + K`-row history, and the thread it would spin is competing for cores with
/// the SDK background thread that must deliver the very updates being waited on — on a loaded host a
/// hot loop could manufacture the timeout it is supposed to detect. This is the backoff loop the
/// sleep policy admits, not a fixed delay standing in for coordination: the exit is the condition,
/// and the deadline is still absolute.
fn await_convergence(
    client: &ConnectedClient,
    expected: &CurrentState,
    expected_history: &History,
) -> Result<()> {
    let deadline = Instant::now() + CONVERGENCE_TIMEOUT;
    loop {
        if registry_state(&client.read_control_registry_all_view()) == *expected
            && activity_state(&client.read_control_activity_latest_by_control_view()) == *expected
            && registry_state(&client.read_control_registry()) == *expected
            && client.read_control_activity().len() == expected_history.len()
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "the subscriber caches did not converge on the post-repeat state within \
                 {CONVERGENCE_TIMEOUT:?}",
            ));
        }
        std::thread::sleep(CONVERGENCE_POLL_INTERVAL);
    }
}

/// Block for at least `minimum` Arm A row callbacks, then drain whatever else is already queued,
/// returning the whole observed sequence in delivery order.
///
/// **Both halves are load-bearing, for opposite reasons.**
///
/// It must *block* first because the pinned SDK writes the cache, runs `on_applied`, and only then
/// runs the row callbacks — so after a subscription is applied, a purely non-blocking read would
/// race the deliveries and mis-attribute one phase's events to the next.
///
/// It must then *drain* because `minimum` is a lower bound, not the count. A control's replacement
/// may surface as one in-place update or as a delete plus an insert, and stopping at exactly K
/// would silently truncate the second shape to a K-event prefix — reporting ten deletes as if they
/// were the whole story, and turning the recording the spec demands back into the assumption it
/// forbids. For the repeat round the caller's completed reducer calls guarantee every delivery is
/// already queued, so the drain there is exhaustive.
fn collect_events(
    events: &mpsc::Receiver<ViewEventShape>,
    minimum: u64,
    phase: &str,
) -> Result<Vec<ViewEventShape>> {
    let deadline = Instant::now() + CONVERGENCE_TIMEOUT;
    let minimum = usize::try_from(minimum).expect("the phase event count fits usize");
    let mut observed = Vec::with_capacity(minimum);

    while observed.len() < minimum {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match events.recv_timeout(remaining) {
            Ok(event) => observed.push(event),
            Err(waiting) => {
                return Err(anyhow!(
                    "{phase} delivered only [{}] of the at-least-{minimum} expected Arm A row \
                     callbacks before {waiting}",
                    describe(&observed),
                ))
            }
        }
    }
    observed.extend(events.try_iter());

    Ok(observed)
}

/// Exercise each fail-loud precondition and prove it **rolled back**, returning one report line per
/// case.
///
/// Every case reads both tables before and after and requires them logically identical across every
/// column — the subscriber sees typed rows, not storage bytes, so this is the logical half of the
/// spec's byte-for-byte/logically-unchanged requirement, and the report says so. That is the
/// difference between a refusal and a rollback: an error message alone would be satisfied by a
/// reducer that wrote the audit row, failed on the registry, and left the two disagreeing. The
/// caller re-checks whole-composition afterward, which also catches a write delivered too late for
/// the immediate comparison.
fn prove_refusals_roll_back(
    client: &ConnectedClient,
    expected: &CurrentState,
    expected_history: &History,
) -> Result<Vec<String>> {
    let unseeded_control = CONTROL_UUID_BASE + CONTROL_COUNT;
    let fresh_id = HISTORY_ROW_COUNT + CONTROL_COUNT;
    let existing_control = control_uuid(0);
    // Exactly the control's current `last_ts`, not merely something older. The frozen precondition
    // is `ts <= last_ts`, and only the equality case separates it from `ts < last_ts`: a reducer
    // regressed to the strict comparison would refuse any older timestamp just as loudly while
    // wrongly accepting a duplicate one, and a strictly-older probe could never catch it.
    let boundary_ts = activity_ts(repeat_round_id(0));

    // Each case carries the substring its refusal must contain, so the proof is that the message
    // *identifies the offending key or condition* rather than merely that some error arrived. A
    // panicking reducer would trap the instance and deliver "The instance encountered a fatal
    // error", which these assertions reject by construction.
    let cases: Vec<(&str, String, Box<dyn Fn() -> Result<()> + '_>)> = vec![
        (
            "repeat activity for an unknown control",
            format!("no control_registry row with control_uuid={unseeded_control}"),
            Box::new(move || {
                client.record_control_activity(fresh_id, unseeded_control, activity_ts(fresh_id))
            }),
        ),
        (
            "first activity for a control that already exists",
            format!("control_registry already holds control_uuid={existing_control}"),
            Box::new(move || {
                client.record_first_control_activity(
                    fresh_id,
                    existing_control,
                    control_identity(0),
                    activity_ts(fresh_id),
                )
            }),
        ),
        (
            "repeat activity reusing an existing activity id",
            "control_activity already holds id=0".to_string(),
            Box::new(move || {
                client.record_control_activity(0, existing_control, activity_ts(fresh_id))
            }),
        ),
        (
            "repeat activity whose ts equals the recorded last_ts",
            "is not strictly later than the recorded last_ts".to_string(),
            Box::new(move || {
                client.record_control_activity(fresh_id, existing_control, boundary_ts)
            }),
        ),
    ];

    let mut report = Vec::with_capacity(cases.len());
    for (label, expected_message, attempt) in cases {
        let registry_before = client.read_control_registry();
        let activity_before = client.read_control_activity();

        let Err(refusal) = attempt() else {
            return Err(anyhow!(
                "{label}: the reducer succeeded, but this precondition must fail loud",
            ));
        };
        let refusal_text = one_line(&refusal);
        ensure!(
            refusal_text.contains(&expected_message),
            "{label}: the refusal did not identify the offending key or condition. Expected the \
             message to contain {expected_message:?}, got {refusal_text:?}",
        );

        let registry_after = client.read_control_registry();
        let activity_after = client.read_control_activity();
        ensure!(
            sorted_registry(&registry_before) == sorted_registry(&registry_after),
            "{label}: the reducer failed but control_registry changed — the transaction did not \
             roll back",
        );
        ensure!(
            sorted_activity(&activity_before) == sorted_activity(&activity_after),
            "{label}: the reducer failed but control_activity changed — the transaction did not \
             roll back",
        );

        report.push(format!("{label}: {refusal_text}"));
    }

    check_composition(client, expected, expected_history, "after the refusal cases")?;
    Ok(report)
}

/// Registry rows in a canonical order, so a comparison judges contents rather than delivery order.
fn sorted_registry(rows: &[ControlRegistry]) -> Vec<(u64, Identity, Timestamp)> {
    let mut sorted: Vec<_> = rows
        .iter()
        .map(|row| (row.control_uuid, row.user_identity, row.last_ts))
        .collect();
    sorted.sort();
    sorted
}

/// Activity rows in a canonical order, for the same reason.
fn sorted_activity(rows: &[ControlActivity]) -> Vec<(u64, u64, Identity, Timestamp)> {
    let mut sorted: Vec<_> = rows
        .iter()
        .map(|row| (row.id, row.control_uuid, row.user_identity, row.ts))
        .collect();
    sorted.sort();
    sorted
}

/// Flatten an error chain to one line, so the report stays readable.
fn one_line(error: &anyhow::Error) -> String {
    format!("{error:#}").replace('\n', " ")
}

/// Render an observed event sequence in delivery order.
fn describe(events: &[ViewEventShape]) -> String {
    events
        .iter()
        .map(|event| event.label())
        .collect::<Vec<_>>()
        .join(", ")
}
