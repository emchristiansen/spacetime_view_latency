//! Capability reproducer for site 4's admin empty-result question (spec c33f2e51): provision one
//! fresh isolated server, publish the unchanged module, seed `control_activity` rows owned by the
//! measured identity, then subscribe to both `control_activity_sender_view` and
//! `control_activity_empty_view` and assert the first returns those rows while the second returns
//! none.
//!
//! **What this establishes and what it cannot.** Production's `control_activity_view_all` returns
//! its non-admin callers an empty result by filtering on `id == u64::MAX` — always false only
//! because no row happens to hold that id. The module's `control_activity_empty_view` replaces that
//! sentinel with the typed contradiction `row.id.ne(row.id)`, which the type checker already proves
//! expressible. What remained unknown is whether the *server* accepts such a query and applies an
//! empty subscription, and that is a runtime question no inspection can settle. Running this
//! answers exactly it.
//!
//! The sender view is the composition control, and it is what makes a zero-row answer mean
//! anything: it proves the seeded rows exist and are visible to this very caller, so the empty
//! view's emptiness is its predicate's doing rather than an empty table. A query the server refused
//! would fail at the subscription rather than return zero rows, so the two failure modes stay
//! distinguishable.
//!
//! This candidate has no `Cell`/`Run` counterpart in the historical arm machinery, so this driver
//! calls [`ConnectedClient`]'s candidate-specific primitives directly and otherwise mirrors
//! [`crate::entity_owner_smoke`]'s provisioning/teardown discipline exactly. It measures nothing:
//! no Site 4 performance run may begin until the spec freezes an append-only E2 estimand.

use std::path::Path;

use anyhow::{anyhow, ensure, Context, Result};
use spacetimedb_sdk::Timestamp;

use crate::client::connected_client::ConnectedClient;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;

/// Small literal row count sufficient for a capability reproducer — this is a capability question,
/// not a measured ladder.
const SEEDED_ROW_COUNT: u64 = 3;

/// Fixed literal `control_uuid` base for the seeded rows. The uuid is not the object of study; it
/// is fixed so a seeded row is reproducible from the seed alone.
const SEEDED_CONTROL_UUID_BASE: u64 = 9_000;

/// Provision, publish, seed, subscribe, and verify the typed-contradiction empty view end to end,
/// tearing the server down on every exit path — mirrors
/// [`crate::entity_owner_smoke::entity_owner_sender_view_smoke`]'s acquisition/teardown pipeline
/// exactly.
pub(crate) fn control_activity_empty_view_reproducer(
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

/// Connect the measured subscriber, run the reproducer, then disconnect unconditionally —
/// aggregating a check failure with a teardown failure exactly like
/// [`crate::entity_owner_smoke`]'s `drive`.
fn drive(server: &RunningPinnedServer, database_identity: &str) -> Result<()> {
    let server_url = server.listen().client_url();
    let client = ConnectedClient::connect(&server_url, database_identity)
        .context("connecting the measured subscriber")?;

    let outcome = run_reproducer(&client);
    let disconnect = client
        .disconnect()
        .context("disconnecting the measured subscriber");

    match (outcome, disconnect) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(body), Ok(())) => Err(body),
        (Ok(()), Err(teardown)) => Err(teardown),
        (Err(body), Err(teardown)) => Err(anyhow!(
            "the empty-view reproducer failed and disconnecting the client afterward also failed:\n  \
             [1] {body:#}\n  [2] {teardown:#}"
        )),
    }
}

/// Seed rows owned by the measured identity, then read both views: the sender view must return
/// exactly the seeded rows, and the typed-contradiction view must return none.
fn run_reproducer(client: &ConnectedClient) -> Result<()> {
    let user_identity = client.measured_identity();
    let ts = Timestamp::now();

    for i in 0..SEEDED_ROW_COUNT {
        client.insert_control_activity(i, ts, SEEDED_CONTROL_UUID_BASE + i, user_identity)?;
    }

    // The control first: an empty answer from the arm means nothing until the rows are known to be
    // present and visible to this caller.
    let visible = client.subscribe_and_read_control_activity_sender_view()?;
    ensure!(
        visible.len() == SEEDED_ROW_COUNT as usize,
        "control_activity_sender_view returned {} rows, expected the {} seeded for the measured \
         identity — the composition control failed, so the empty view's answer proves nothing",
        visible.len(),
        SEEDED_ROW_COUNT,
    );
    ensure!(
        visible
            .iter()
            .all(|row| row.user_identity == user_identity),
        "control_activity_sender_view returned a row belonging to another identity",
    );

    let empty = client.subscribe_and_read_control_activity_empty_view()?;
    ensure!(
        empty.is_empty(),
        "control_activity_empty_view returned {} rows, expected none — the typed contradiction \
         `row.id.ne(row.id)` did not produce an empty result at runtime",
        empty.len(),
    );

    println!(
        "control_activity_empty_view reproducer: seeded {SEEDED_ROW_COUNT} rows for the measured \
         identity; control `control_activity_sender_view` returned all {} of them, arm \
         `control_activity_empty_view` was accepted by the server and returned 0 rows — the typed \
         contradiction `row.id.ne(row.id)` yields a genuinely empty view result without a sentinel",
        visible.len(),
    );

    Ok(())
}
