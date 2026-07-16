//! Execute one arm-or-control run: seed the dataset and check the initial result set.

use anyhow::{Context, Result};

use crate::client::connected_client::ConnectedClient;
use crate::dataset::seed_plan::SeedPlan;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::plan::run::Run;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::roles::role_identities::RoleIdentities;

/// Execute one [`Run`] against the provisioned `server`.
///
/// Pre-write milestone (no measurement): connect the measured subscriber capturing its
/// server-issued identity, resolve the typed role identities, seed the deterministic
/// regime-specific dataset through confirmed reducers, assert `(viewer, message_uuid)`
/// pair uniqueness, subscribe to the arm view (or matched control table) *after* seeding,
/// and assert the initial cache result set equals the seed-derived expected set. Because
/// every C/E/F′ arm's expected set is exactly the measured slice's Chronicle rows, this
/// per-run equality transitively certifies their cross-arm equivalence without a co-located
/// tri-subscription run. The post-write correctness check is deferred to the measured-write
/// milestone.
pub(crate) fn execute_run(
    server: &RunningPinnedServer,
    manifest: &ValidatedRunManifest,
    run: Run,
) -> Result<()> {
    let server_url = server.listen().client_url();
    let database_identity = manifest.database_identity().identity().to_hex().to_string();

    let client = ConnectedClient::connect(&server_url, &database_identity)
        .context("connecting the measured subscriber")?;

    let identities = RoleIdentities::resolve(
        client.measured_identity(),
        manifest.schedule_seed(),
        run.cell(),
    )?;

    let plan = SeedPlan::resolve(run, &identities);
    plan.assert_unique_pairs()?;
    client
        .seed(&plan.operations())
        .context("seeding the deterministic dataset")?;

    let target = SubscribedTable::from_run(run);
    let observed = client.subscribe_and_read(target)?;
    let expected = target.expected(&plan);
    observed
        .assert_equals(&expected)
        .with_context(|| format!("initial result-set correctness for {run:?}"))?;

    println!(
        "run correctness OK: {run:?} — subscribed {:?}, {} rows match the seed-derived \
         expected set",
        target.subscription_sql(),
        observed.len(),
    );

    client.disconnect()?;
    Ok(())
}
