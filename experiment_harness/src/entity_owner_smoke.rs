//! Direct Smoke-stage execution for the `EntityOwnerSenderView` candidate (spec c33f2e51):
//! provision one fresh isolated server, publish the unchanged module, seed a handful of
//! `entity_owner` rows for the measured identity and a distinct non-owner identity, subscribe to
//! `entity_owner_sender_view`, and assert only the measured identity's own rows come back — the
//! spec's "one plumbing and semantic sanity execution" for this candidate.
//!
//! This candidate has no `Cell`/`Run` counterpart in the historical arm machinery (that ontology
//! is fixed to the `Message`/`ChronicleMessage` mechanism study and is preserved, not
//! reinterpreted), so this driver calls [`ConnectedClient`]'s candidate-specific primitives
//! directly. It otherwise mirrors [`crate::quick_run::quick_run`]'s provisioning/teardown
//! discipline exactly.

use std::path::Path;
use std::time::Instant;

use anyhow::{anyhow, ensure, Context, Result};
use spacetimedb_sdk::Identity;

use crate::client::connected_client::ConnectedClient;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::params::EXPERIMENT_ISSUER;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;

/// Fixed literal subject deriving the smoke path's distinct non-owner identity, analogous to
/// `quick_run::QUICK_RUN_GROWTH_SUBJECT`.
const ENTITY_OWNER_SMOKE_OTHER_OWNER_SUBJECT: &str = "entity-owner-smoke-other-owner";

/// Small literal row counts sufficient for a plumbing/semantic sanity check — this is Smoke, not
/// a measured ladder.
const OWNED_ROW_COUNT: u64 = 3;
const OTHER_OWNER_ROW_COUNT: u64 = 3;

/// Provision, publish, seed, subscribe, and verify the `EntityOwnerSenderView` candidate end to
/// end, tearing the server down on every exit path — mirrors
/// [`crate::quick_run::quick_run`]'s acquisition/teardown pipeline exactly.
pub(crate) fn entity_owner_sender_view_smoke(listen: ListenAddress, module_wasm: &Path) -> Result<()> {
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

/// Connect the measured subscriber, run the smoke sanity check, then disconnect unconditionally —
/// aggregating a check failure with a teardown failure exactly like [`crate::quick_run::quick_run`]'s
/// `drive`.
fn drive(server: &RunningPinnedServer, database_identity: &str) -> Result<()> {
    let server_url = server.listen().client_url();
    let client = ConnectedClient::connect(&server_url, database_identity)
        .context("connecting the measured subscriber")?;

    let outcome = run_smoke(&client);
    let disconnect = client
        .disconnect()
        .context("disconnecting the measured subscriber");

    match (outcome, disconnect) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(body), Ok(())) => Err(body),
        (Ok(()), Err(teardown)) => Err(teardown),
        (Err(body), Err(teardown)) => Err(anyhow!(
            "the smoke run failed and disconnecting the client afterward also failed:\n  \
             [1] {body:#}\n  [2] {teardown:#}"
        )),
    }
}

/// Seed rows owned by the measured identity and by a distinct non-owner identity, subscribe to
/// `entity_owner_sender_view`, and assert the returned rows are exactly — and only — the measured
/// identity's own rows: the security gate ("another user cannot subscribe to an exact key they do
/// not own") applied to this candidate's Smoke stage.
fn run_smoke(client: &ConnectedClient) -> Result<()> {
    let owner = client.measured_identity();
    let other_owner = Identity::from_claims(EXPERIMENT_ISSUER, ENTITY_OWNER_SMOKE_OTHER_OWNER_SUBJECT);

    let start = Instant::now();
    for i in 0..OWNED_ROW_COUNT {
        client.insert_entity_owner(i, owner, format!("owned-{i}"))?;
    }
    for i in 0..OTHER_OWNER_ROW_COUNT {
        client.insert_entity_owner(OWNED_ROW_COUNT + i, other_owner, format!("other-{i}"))?;
    }
    let seed_elapsed = start.elapsed();

    let rows = client.subscribe_and_read_entity_owner_sender_view()?;

    ensure!(
        rows.len() == OWNED_ROW_COUNT as usize,
        "entity_owner_sender_view returned {} rows, expected exactly the {} owned by the measured identity",
        rows.len(),
        OWNED_ROW_COUNT,
    );
    ensure!(
        rows.iter().all(|row| row.owner == owner),
        "entity_owner_sender_view leaked a row not owned by the measured identity — security gate violated",
    );

    println!(
        "entity_owner_sender_view smoke: seeded {} rows ({} owned, {} other-owner) in {:?}; \
         subscribed view returned exactly the {} owned rows, none leaked",
        OWNED_ROW_COUNT + OTHER_OWNER_ROW_COUNT,
        OWNED_ROW_COUNT,
        OTHER_OWNER_ROW_COUNT,
        seed_elapsed,
        rows.len(),
    );

    Ok(())
}
