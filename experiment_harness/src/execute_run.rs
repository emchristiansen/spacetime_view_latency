//! Execute one arm-or-control run: seed the dataset and check the initial result set.

use anyhow::{anyhow, Context, Result};

use crate::client::connected_client::ConnectedClient;
use crate::dataset::seed_plan::SeedPlan;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::plan::control_table::ControlTable;
use crate::plan::run::Run;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::roles::role_identities::RoleIdentities;

/// Execute one [`Run`] against the provisioned `server`.
///
/// Pre-write milestone (no measurement): connect the measured subscriber capturing its
/// server-issued identity, then run every correctness check against the live client and
/// disconnect on *every* path before returning. Because a check failure must not leave the
/// client's background message thread alive while the server is torn down out from under it,
/// [`run_checks`] is separated so its early-return failures still flow through the
/// unconditional `disconnect` here — mirroring the provisioning driver's teardown discipline.
///
/// The checks (see [`run_checks`]): resolve the typed role identities, seed the deterministic
/// regime-specific dataset through confirmed reducers, read the live `message_visibility`
/// rows back and assert `(viewer, message_uuid)` pair uniqueness against actual server state
/// (Chronicle family only), subscribe to the arm view (or matched control table) *after*
/// seeding, and assert the initial cache result set equals the seed-derived expected set.
/// Because every C/E/F′ arm's expected set is exactly the measured slice's Chronicle rows,
/// this per-run equality transitively certifies their cross-arm equivalence without a
/// co-located tri-subscription run. The post-write correctness check is deferred to the
/// measured-write milestone.
pub(crate) fn execute_run(
    server: &RunningPinnedServer,
    manifest: &ValidatedRunManifest,
    run: Run,
) -> Result<()> {
    let server_url = server.listen().client_url();
    let database_identity = manifest.database_identity().identity().to_hex().to_string();

    let client = ConnectedClient::connect(&server_url, &database_identity)
        .context("connecting the measured subscriber")?;

    // Run the checks, then disconnect unconditionally and aggregate: a failed check and a
    // failed teardown are both surfaced, and the run only succeeds if both did.
    let outcome = run_checks(&client, manifest, run);
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

/// Seed the deterministic dataset and assert its initial result set against the live
/// `client`. Kept separate from [`execute_run`] so any early-return failure still flows
/// through the caller's unconditional client teardown rather than a bare `?` that would
/// strand the live connection thread.
fn run_checks(client: &ConnectedClient, manifest: &ValidatedRunManifest, run: Run) -> Result<()> {
    let identities = RoleIdentities::resolve(
        client.measured_identity(),
        manifest.schedule_seed(),
        run.cell(),
    )?;

    let plan = SeedPlan::resolve(run, &identities);
    client
        .seed(&plan.operations())
        .context("seeding the deterministic dataset")?;

    // Defense in depth: read the live `message_visibility` rows back and assert the seeded
    // `(viewer, message_uuid)` pairs are unique against actual server state. Only the
    // Chronicle family seeds visibility pairs; the single-table `message` family has none.
    if plan.family() == ControlTable::ChronicleMessage {
        let visibility = client
            .read_back_visibility()
            .context("reading back the seeded message_visibility rows")?;
        visibility
            .assert_unique_pairs(plan.chronicle_pair_count())
            .context("seeded (viewer, message_uuid) pair uniqueness")?;
    }

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
    Ok(())
}
