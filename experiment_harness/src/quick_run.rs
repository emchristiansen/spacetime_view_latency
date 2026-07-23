//! The quick-and-dirty nine-cell/eighteen-run driver: literal deterministic setup and dose rows,
//! in-memory latency summaries, direct printing. No `Schedule`/`BlockRun`, no manifest lifecycle,
//! no `ObservationSink`/NDJSON — results-first reconnaissance only (see the governing spec's
//! "Results-First Revision").

use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use spacetimedb_sdk::Identity;

use crate::client::connected_client::ConnectedClient;
use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::{BATCH_DELAY_MS, EXPERIMENT_ISSUER};
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::plan::run::Run;
use crate::plan::table_scoped_arm::TableScopedArm;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;
use crate::roles::role_identities::RoleIdentities;

/// Fixed literal subject deriving the quick path's non-connecting growth identity `G`. Each quick
/// run provisions its own fresh isolated server, so no per-run domain separation is needed.
const QUICK_RUN_GROWTH_SUBJECT: &str = "quick-run-growth";

/// The nine cells in the spec's Execution Order: F and F′ under both growth regimes first (the
/// highest-risk wiring — both growth regimes plus the corrected Chronicle order), then A–E.
/// `smoke_only` selects just the first four.
pub(crate) fn quick_run_cells(smoke_only: bool) -> Vec<Cell> {
    let smoke = vec![
        Cell::KeyScopedUnrelated(KeyScopedArm::PointFilter),
        Cell::KeyScopedUnrelated(KeyScopedArm::PointSemijoin),
        Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter),
        Cell::KeyScopedOwnSlice(KeyScopedArm::PointSemijoin),
    ];
    if smoke_only {
        return smoke;
    }
    let rest = vec![
        Cell::TableScopedUnrelated(TableScopedArm::ProceduralRange),
        Cell::TableScopedUnrelated(TableScopedArm::QueryFull),
        Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoin),
        Cell::TableScopedUnrelated(TableScopedArm::QueryFullPk),
        Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoinPk),
    ];
    smoke.into_iter().chain(rest).collect()
}

/// Provision one fresh isolated server, publish the unchanged module, run `run` against it end to
/// end, and tear the server down on every exit path.
///
/// Mirrors [`crate::provision::provision::provision_run_resources`]'s acquisition pipeline and
/// [`crate::execute_run::execute_run`]'s connect/act/disconnect teardown discipline directly,
/// without minting a `RunCoordinate` or `ValidatedRunManifest`: this path never schedules,
/// records, or serializes a run, so it needs neither.
pub(crate) fn quick_run(listen: ListenAddress, module_wasm: &Path, run: Run) -> Result<()> {
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

    let outcome = drive(resources.server(), &database_identity, run);

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

/// Connect the measured subscriber, run the measured ladder, then disconnect unconditionally —
/// aggregating a measurement failure with a teardown failure exactly like
/// [`crate::execute_run::execute_run`].
fn drive(server: &RunningPinnedServer, database_identity: &str, run: Run) -> Result<()> {
    let server_url = server.listen().client_url();
    let client = ConnectedClient::connect(&server_url, database_identity)
        .context("connecting the measured subscriber")?;

    let outcome = run_measured(&client, run);
    let disconnect = client
        .disconnect()
        .context("disconnecting the measured subscriber");

    match (outcome, disconnect) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(body), Ok(())) => Err(body),
        (Ok(()), Err(teardown)) => Err(teardown),
        (Err(body), Err(teardown)) => Err(anyhow!(
            "the run failed and disconnecting the client afterward also failed:\n  \
             [1] {body:#}\n  [2] {teardown:#}"
        )),
    }
}

/// Seed the literal pinned/background rows, subscribe and verify the pre-dose baseline, then
/// execute all ten cumulative doses: each dose's writes are timed in memory, its post-dose cache
/// state is checked, and its per-dose table row is printed immediately.
fn run_measured(client: &ConnectedClient, run: Run) -> Result<()> {
    let growth = Identity::from_claims(EXPERIMENT_ISSUER, QUICK_RUN_GROWTH_SUBJECT);
    let identities = RoleIdentities::from_identities(client.measured_identity(), growth)
        .context("resolving quick-run role identities")?;

    let dataset = CampaignDataset::resolve(run, &identities);

    client
        .seed(&dataset.background_operations())
        .context("seeding the unmeasured pinned/background slice")?;

    let target = SubscribedTable::from_run(run);
    client
        .subscribe(target)
        .context("subscribing to the run's target")?;

    let initial = client.read_current(target);
    let expected_initial = target.expected_initial(&dataset);
    initial
        .assert_equals(&expected_initial)
        .context("initial (pre-dose) result-set correctness")?;

    println!(
        "quick-run {:?} {:?}: initial set OK ({} rows)",
        run.cell(),
        run.role(),
        initial.len(),
    );

    for &dose in &DoseIndex::ALL {
        let batch = DoseBatch::new(&dataset, dose);
        let operations = batch.operations();

        let raw = client
            .measure_dose(&operations)
            .with_context(|| format!("measuring dose {}", dose.get()))?;

        let observed = client.read_current(target);
        let expected = target.expected_through_dose(&dataset, dose);
        observed
            .assert_equals(&expected)
            .with_context(|| format!("post-dose {} result-set correctness", dose.get()))?;

        let p50_nanos = median_nanos(&raw);

        println!(
            "  dose {:>2}  n_driving={:<7}  n_samples={:<5}  p50={:?}",
            dose.get(),
            batch.cumulative_driving_rows(),
            operations.len(),
            Duration::from_nanos(p50_nanos.min(u64::MAX as u128) as u64),
        );

        thread::sleep(Duration::from_millis(BATCH_DELAY_MS));
    }

    Ok(())
}

/// The R-1 nearest-rank median (p50) of one dose's confirmed round-trip latencies, computed
/// directly from an in-memory sorted copy — no `observation::latency_summary` dependency.
fn median_nanos(raw: &RawLatencies) -> u128 {
    let mut nanos: Vec<u128> = raw.samples().iter().map(|sample| sample.nanos()).collect();
    nanos.sort_unstable();
    let rank = (nanos.len() + 1) / 2 - 1;
    nanos[rank]
}
