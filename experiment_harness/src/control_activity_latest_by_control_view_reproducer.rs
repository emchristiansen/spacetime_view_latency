//! Bounded capability probe for site 4's discovery comparator (spec c33f2e51): provision one fresh
//! isolated server, publish the unchanged module, seed a small `control_activity` history, and prove
//! the four frozen criteria P1–P4 for a procedural view that computes latest-per-`control_uuid` over
//! that unindexed table.
//!
//! **P1 is already decided, and it did not go the preferred way.** The view was first written
//! against `spacetimedb::Local {}`, which is `#[non_exhaustive]` and so unconstructible from a
//! foreign crate; the pinned compiler rejected it with `E0639`. The authorized substitution — the
//! macro-generated in-crate `control_activity__TableHandle {}` plus `Table::iter` — compiled, and
//! **that is the route P2–P4 exercise.** The module's view carries the full source argument and its
//! version-coupling cost; it is not restated here.
//!
//! **The remaining criteria, frozen before execution.** P2 exact-hash publication to a fresh pinned
//! server succeeds. P3 the subscription applies and its cache holds exactly the expected
//! latest-per-control set. P4 a strictly later row inserted for one control *after* subscribing is
//! reflected in that same subscription's cache.
//!
//! **P4 is the criterion that matters and it cannot be skipped.** An imperative scan is invisible to
//! the machinery that learns a view's dependencies from the query it compiles, so the dangerous
//! outcome is not rejection: it is a view that materializes correctly, satisfies P2–P3, and is then
//! never recomputed when the table it scanned changes — silently serving a stale snapshot. Passing
//! P2–P3 and failing P4 is a correctness failure, not a capability, and retires the comparator
//! rather than establishing anything.
//!
//! **What a pass does not establish.** Capability only. Nothing here is timed, no performance run
//! may use this view, and whether the composition-matched comparator is reinstated is a decision
//! this probe returns to Control. The raw `control_activity` table is seeded through the existing
//! [`insert_control_activity`](crate::client::connected_client::ConnectedClient::insert_control_activity)
//! reducer and no table, reducer, or index is added anywhere.
//!
//! Mirrors [`crate::control_activity_empty_view_reproducer`]'s provisioning/teardown discipline
//! exactly, which in turn mirrors [`crate::entity_owner_smoke`]'s.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, ensure, Context, Result};
use spacetimedb_sdk::{Identity, Timestamp};

use crate::client::connected_client::ConnectedClient;
use crate::client::view_event_shape::ViewEventShape;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::bindings::ControlActivity;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;

/// Distinct controls in the seeded history — the spec's frozen `K`. Deliberately tiny: this is a
/// capability question, not a measured ladder, and a larger history would prove nothing further
/// while making a failure harder to read.
const SEEDED_CONTROL_COUNT: u64 = 3;

/// Total seeded history rows — the spec's frozen `N`, four per control. Four rather than two so a
/// control's latest row is neither its first nor its second, which a comparator that silently kept
/// the wrong end would satisfy by accident.
const SEEDED_ROW_COUNT: u64 = 12;

/// History rows per control, derived rather than declared so the two frozen constants above cannot
/// disagree with the seeding loop about their own composition.
const ROWS_PER_CONTROL: u64 = SEEDED_ROW_COUNT / SEEDED_CONTROL_COUNT;

/// Fixed literal `control_uuid` base for the seeded controls. Not the object of study; fixed so a
/// seeded row is reproducible from the seed alone.
const SEEDED_CONTROL_UUID_BASE: u64 = 9_000;

/// Fixed literal base timestamp, in microseconds since the Unix epoch, so the seeded state is
/// reproducible rather than clock-dependent — the discipline
/// [`crate::control_activity_empty_view_reproducer`] was corrected to follow. Unlike that
/// reproducer, this one's view *does* read `ts`, so the seed must also make the ordering
/// unambiguous, which [`activity_ts`] does.
const SEEDED_TS_BASE_MICROS: i64 = 1_700_000_000_000_000;

/// Microseconds between consecutive seeded rows. One row's `ts` is one step past the previous
/// **global** row's, not the previous row of its own control, so every timestamp in the seed is
/// globally unique and no two rows can tie under any comparator.
const SEEDED_TS_STEP_MICROS: i64 = 1_000;

/// Wait budget for the P4 invalidation to reach the subscriber cache after its insert is confirmed.
///
/// Matched to the harness's other single-round-trip budgets rather than shortened: a budget too
/// tight would report a slow-but-correct view as an uninvalidated one, which is the exact confusion
/// this probe exists to avoid. Elapsing means no invalidation arrived, which is a P4 failure and a
/// real result — never a retryable infrastructure fault.
const INVALIDATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Provision, publish, seed, subscribe, and verify P1–P4 end to end, tearing the server down on
/// every exit path — mirrors [`crate::control_activity_empty_view_reproducer`]'s
/// acquisition/teardown pipeline exactly.
pub(crate) fn control_activity_latest_by_control_view_reproducer(
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

    // P2 is exactly this call succeeding: the pinned server accepted a module carrying a procedural
    // view whose body scans an unindexed table through the generated typed handle, with an explicit
    // primary key on a non-key column of the returned row type. A rejection surfaces here as a
    // publication error, which is the P2 evidence.
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

/// Connect the subscriber, run the probe, then disconnect unconditionally — aggregating a check
/// failure with a teardown failure exactly like
/// [`crate::control_activity_empty_view_reproducer`]'s `drive`.
fn drive(server: &RunningPinnedServer, database_identity: &str) -> Result<()> {
    let server_url = server.listen().client_url();
    let client = ConnectedClient::connect(&server_url, database_identity)
        .context("connecting the probe subscriber")?;

    let outcome = run_probe(&client);
    let disconnect = client
        .disconnect()
        .context("disconnecting the probe subscriber");

    match (outcome, disconnect) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(body), Ok(())) => Err(body),
        (Ok(()), Err(teardown)) => Err(teardown),
        (Err(body), Err(teardown)) => Err(anyhow!(
            "the latest-per-control capability probe failed and disconnecting the client afterward \
             also failed:\n  [1] {body:#}\n  [2] {teardown:#}"
        )),
    }
}

/// Seed the history, subscribe once, check P3, then insert one strictly later row and check P4 on
/// that same live subscription.
fn run_probe(client: &ConnectedClient) -> Result<()> {
    let user_identity = client.measured_identity();
    seed_history(client, user_identity)?;

    // Registered *before* the subscription, so nothing the subscription itself delivers can slip
    // past unobserved — the ordering the campaign's delivery counters use for the same reason.
    let (events_tx, events_rx) = mpsc::channel::<ViewEventShape>();
    client.observe_control_activity_latest_by_control_view(events_tx);

    client
        .subscribe_control_activity_latest_by_control_view()
        .context("subscribing to control_activity_latest_by_control_view (P3)")?;
    let materialized = client.read_control_activity_latest_by_control_view();
    check_latest_set(&materialized, &expected_latest(user_identity), "P3")?;

    let snapshot_events = await_snapshot_events(&events_rx)?;

    let (later_id, later_control_uuid, later_ts) = later_activity();
    client
        .insert_control_activity(later_id, later_ts, later_control_uuid, user_identity)
        .context("inserting the later activity row (P4)")?;

    let expected_after = expected_latest_after_later_activity(user_identity);
    let invalidation = await_invalidation(client, &events_rx, &expected_after)?;

    println!(
        "control_activity_latest_by_control_view probe: P1 the preferred `Local {{}}` route FAILED \
         to compile (E0639, `Local` is #[non_exhaustive]) and the generated typed \
         `control_activity__TableHandle {{}}` route compiled — the latter is what this run \
         exercises; P2 published; P3 the subscription applied and returned exactly {} \
         latest-per-control rows over {} seeded history rows across {} controls; P4 a strictly \
         later row for control {} was reflected in the same live subscription. Initial snapshot \
         delivered [{}]; the invalidation delivered [{}] in that order. The pinned release \
         therefore both materializes and maintains a procedural view that scans an unindexed table \
         through the generated typed handle. Capability only: this proves neither acceptable \
         version-coupling risk nor performance.",
        materialized.len(),
        SEEDED_ROW_COUNT,
        SEEDED_CONTROL_COUNT,
        later_control_uuid,
        describe(&snapshot_events),
        describe(&invalidation),
    );

    Ok(())
}

/// Seed `SEEDED_ROW_COUNT` history rows through the module's existing history-only insertion
/// reducer, `ROWS_PER_CONTROL` per control.
///
/// The existing reducer deliberately: this probe adds no reducer, and the registry candidate's
/// atomic writer contract is a separate, later milestone whose invariant this seeding neither holds
/// nor claims.
fn seed_history(client: &ConnectedClient, user_identity: Identity) -> Result<()> {
    for id in 0..SEEDED_ROW_COUNT {
        client.insert_control_activity(
            id,
            activity_ts(id),
            SEEDED_CONTROL_UUID_BASE + (id % SEEDED_CONTROL_COUNT),
            user_identity,
        )?;
    }
    Ok(())
}

/// The timestamp for seeded row `id` — strictly increasing in `id` across the whole seed, so every
/// timestamp is globally unique and each control's latest row is unambiguously its highest `id`.
fn activity_ts(id: u64) -> Timestamp {
    let offset = i64::try_from(id).expect("the seeded row count fits i64");
    Timestamp::from_micros_since_unix_epoch(SEEDED_TS_BASE_MICROS + offset * SEEDED_TS_STEP_MICROS)
}

/// Round-robin seeding assigns control `c` the rows `c, c + K, c + 2K, …`, so its latest is the
/// largest such id below `SEEDED_ROW_COUNT`.
fn latest_seeded_id(control_index: u64) -> u64 {
    control_index + SEEDED_CONTROL_COUNT * (ROWS_PER_CONTROL - 1)
}

/// The latest-per-control set the view must return after seeding — computed from the seed's own
/// rule rather than read back from the server, so P3 compares the view against an independently
/// derived expectation rather than against itself.
fn expected_latest(user_identity: Identity) -> Vec<ControlActivity> {
    (0..SEEDED_CONTROL_COUNT)
        .map(|control_index| {
            let id = latest_seeded_id(control_index);
            ControlActivity {
                id,
                ts: activity_ts(id),
                control_uuid: SEEDED_CONTROL_UUID_BASE + control_index,
                user_identity,
            }
        })
        .collect()
}

/// The one later row inserted after subscribing: `(id, control_uuid, ts)`.
///
/// Its id and timestamp both continue the seed's sequence, so it is strictly later than every
/// seeded row under either ordering, and it targets the **first** control — whose seeded latest is
/// not the global latest, so a view that merely re-reported the table's maximum row would fail P4
/// rather than pass it by coincidence.
fn later_activity() -> (u64, u64, Timestamp) {
    let id = SEEDED_ROW_COUNT;
    (id, SEEDED_CONTROL_UUID_BASE, activity_ts(id))
}

/// The latest-per-control set the view must return once the later row is committed: the seeded
/// expectation with the first control's row replaced.
fn expected_latest_after_later_activity(user_identity: Identity) -> Vec<ControlActivity> {
    let (id, control_uuid, ts) = later_activity();
    let mut expected = expected_latest(user_identity);
    let replaced = expected
        .iter_mut()
        .find(|row| row.control_uuid == control_uuid)
        .expect("the later activity targets a seeded control");
    *replaced = ControlActivity {
        id,
        ts,
        control_uuid,
        user_identity,
    };
    expected
}

/// Block until the initial snapshot's own row callbacks have all arrived, returning their shapes.
///
/// **This must block rather than drain what happens to be queued.** The pinned SDK writes the cache,
/// runs `on_applied`, and only then runs the row callbacks, so the subscription barrier returning
/// proves the cache is populated but proves nothing about the callbacks. A non-blocking drain here
/// races them: any snapshot event still in flight would be read by [`await_invalidation`] instead
/// and reported as part of the change under test. P4's answer would survive that — that wait
/// re-reads the cache after every event and only returns on a match — but the spec requires the
/// *observed event shape* be recorded, and a mis-attributed snapshot insert corrupts exactly that.
///
/// The count is known exactly: the view returns one row per seeded control, so the snapshot delivers
/// [`SEEDED_CONTROL_COUNT`] inserts. Waiting for a known count is what makes the boundary between
/// the two phases sharp instead of timing-dependent.
fn await_snapshot_events(events: &mpsc::Receiver<ViewEventShape>) -> Result<Vec<ViewEventShape>> {
    let deadline = Instant::now() + INVALIDATION_TIMEOUT;
    let expected = usize::try_from(SEEDED_CONTROL_COUNT).expect("the seeded control count fits usize");
    let mut observed = Vec::with_capacity(expected);

    while observed.len() < expected {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match events.recv_timeout(remaining) {
            Ok(event) => observed.push(event),
            Err(waiting) => {
                return Err(anyhow!(
                    "the initial snapshot delivered only [{}] of the {expected} expected row \
                     callbacks before {waiting} — the probe cannot separate snapshot delivery from \
                     the invalidation under test, so P4's observed shape would be unattributable",
                    describe(&observed),
                ))
            }
        }
    }

    Ok(observed)
}

/// Block until the live subscription's cache equals `expected`, returning the event shapes observed
/// on the way there — P4.
///
/// Re-reads the cache after **every** delivered event rather than after a fixed number, because the
/// change may arrive as one update or as a delete plus an insert and the probe may not presume
/// which. The deadline is absolute, so a change delivered in several events cannot extend the wait
/// past its budget.
///
/// An elapsed deadline is the P4 failure, reported with what did arrive: the view materialized but
/// was never invalidated by the write it depends on. That is a correctness result about the pinned
/// release, not an infrastructure fault, so it fails loud here rather than retrying.
fn await_invalidation(
    client: &ConnectedClient,
    events: &mpsc::Receiver<ViewEventShape>,
    expected: &[ControlActivity],
) -> Result<Vec<ViewEventShape>> {
    let deadline = Instant::now() + INVALIDATION_TIMEOUT;
    let mut observed = Vec::new();

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match events.recv_timeout(remaining) {
            Ok(event) => {
                observed.push(event);
                let current = client.read_control_activity_latest_by_control_view();
                if latest_set(&current) == latest_set(expected) {
                    return Ok(observed);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let current = client.read_control_activity_latest_by_control_view();
                return Err(anyhow!(
                    "P4 FAILED: no invalidation reached the subscriber cache within \
                     {INVALIDATION_TIMEOUT:?} of the later row being committed. Observed [{}]; the \
                     view cache still holds {:?} but should hold {:?}. The view materialized \
                     correctly and is not maintained: an imperative scan performed inside the view \
                     body is invisible to the subscription's read-set tracking, so the comparator \
                     is Retired(UntrackedReadSet) rather than capable",
                    describe(&observed),
                    latest_set(&current),
                    latest_set(expected),
                ));
            }
            // Both senders live in callbacks the connection owns, so a disconnect means the SDK
            // dropped them rather than that the wait elapsed — a distinct failure from P4.
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(anyhow!(
                    "the view event channel disconnected while awaiting invalidation, after \
                     observing [{}] — the SDK dropped the row callbacks without delivering the \
                     change, which is not a P4 result",
                    describe(&observed),
                ))
            }
        }
    }
}

/// Compare a view result against an independently derived expectation by value, multiplicity, and
/// cardinality, naming the criterion in any failure.
fn check_latest_set(actual: &[ControlActivity], expected: &[ControlActivity], criterion: &str) -> Result<()> {
    ensure!(
        actual.len() == expected.len(),
        "{criterion} FAILED: control_activity_latest_by_control_view returned {} rows, expected \
         exactly {} — one per seeded control",
        actual.len(),
        expected.len(),
    );
    ensure!(
        latest_set(actual) == latest_set(expected),
        "{criterion} FAILED: control_activity_latest_by_control_view returned {:?}, expected {:?}",
        latest_set(actual),
        latest_set(expected),
    );
    Ok(())
}

/// A view result as an order-independent map from `control_uuid` to **every other column** of that
/// control's row.
///
/// The value carries `user_identity` as well as `(id, ts)` because the spec requires equality by
/// *value*, and a map keyed on the control that compared only the ordering columns would accept a
/// row attributed to the wrong identity — the one column a discovery view exists to carry
/// correctly. `ControlActivity` has exactly these four columns, so keying on one and comparing the
/// other three compares whole rows.
///
/// A `BTreeMap` rather than a sorted `Vec` so a duplicated `control_uuid` cannot compare equal to a
/// correct result of the same length: the view's whole contract is one row per control, and key
/// collapsing is exactly the violation that must not pass silently — which is why cardinality is
/// asserted separately, before this comparison, rather than inferred from it.
fn latest_set(rows: &[ControlActivity]) -> BTreeMap<u64, (u64, Timestamp, Identity)> {
    rows.iter()
        .map(|row| (row.control_uuid, (row.id, row.ts, row.user_identity)))
        .collect()
}

/// Render an observed event sequence for the report, in the order the SDK delivered it.
fn describe(events: &[ViewEventShape]) -> String {
    events
        .iter()
        .map(|event| event.label())
        .collect::<Vec<_>>()
        .join(", ")
}
