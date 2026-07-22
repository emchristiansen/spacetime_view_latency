//! SpacetimeDB 2.6.1 view read-set experiment harness.
//!
//! Phase 2 (in progress): the typed plan model, the version-selection/assertion and
//! immutable run-manifest types, and the server-provisioning capabilities/driver
//! (start/publish/shutdown of the `/proc`-proven pinned standalone) are present but not yet
//! verified end-to-end. Seeding, subscription, measurement, and deterministic randomization
//! remain stubbed. Several typed model members are consumed only in later phases, so
//! unused-code is allowed crate-wide for the skeleton.
#![allow(dead_code)]

mod analysis;
mod campaign;
mod client;
mod dataset;
mod execute_run;
mod manifest;
mod module_artifact;
mod observation;
mod params;
mod plan;
mod provision;
mod roles;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::campaign::run_campaign;
use crate::campaign::CampaignOutcome;
use crate::campaign::CampaignPartialPath;
use crate::campaign::CampaignRoot;
use crate::execute_run::execute_run;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::output_path::OutputPath;
use crate::plan::run_role::RunRole;
use crate::plan::schedule::Schedule;
use crate::provision::provision::{provision_and_run, provision_run};

/// Command-line interface. The mode is an explicit subcommand — no implicit default or
/// boolean flag selects effectful behavior.
#[derive(Parser)]
#[command(about = "SpacetimeDB 2.6.1 view read-set experiment harness")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the preregistered plan and exit, without provisioning or measuring.
    Plan,
    /// Provision one isolated server for a single valid scheduled run, emit its validated run
    /// manifest as JSON, and stop. This is the effectful provisioning smoke command; it does
    /// not measure.
    Provision {
        /// Explicit `host:port` listen address for the isolated experiment standalone.
        #[arg(long)]
        server: String,
        /// Path to the built module WASM whose bytes are hash-verified before publication.
        #[arg(long)]
        module_wasm: PathBuf,
        /// Explicit seed recorded in the manifest for deterministic scheduling.
        #[arg(long)]
        seed: u64,
        /// Index of the plan cell to provision, validated against the schedule.
        #[arg(long)]
        cell_index: usize,
        /// 0-based repetition-block index within that cell, validated against the schedule.
        #[arg(long)]
        block: u32,
        /// Whether to provision the arm run or its matched control run.
        #[arg(long, value_enum)]
        role: RunRole,
        /// Git worktree checked out against the harness's own embedded build commit. Phase 1:
        /// accepted and threaded but not yet compared (Phase 2: `BuildProvenance`'s
        /// `EmbeddedHarnessCommit` checkout verification).
        #[arg(long)]
        harness_checkout_root: PathBuf,
    },
    /// Provision one isolated server for a single valid scheduled run, then seed its
    /// deterministic dataset and assert the initial subscribed result set equals the
    /// seed-derived expected set. Does not measure latency (pre-write correctness only).
    Run {
        /// Explicit `host:port` listen address for the isolated experiment standalone.
        #[arg(long)]
        server: String,
        /// Path to the built module WASM whose bytes are hash-verified before publication.
        #[arg(long)]
        module_wasm: PathBuf,
        /// Explicit seed recorded in the manifest and used to derive role identities.
        #[arg(long)]
        seed: u64,
        /// Index of the plan cell to run, validated against the schedule.
        #[arg(long)]
        cell_index: usize,
        /// 0-based repetition-block index within that cell, validated against the schedule.
        #[arg(long)]
        block: u32,
        /// Whether to run the arm or its matched control.
        #[arg(long, value_enum)]
        role: RunRole,
        /// Git worktree checked out against the harness's own embedded build commit. Phase 1:
        /// accepted and threaded but not yet compared (Phase 2: `BuildProvenance`'s
        /// `EmbeddedHarnessCommit` checkout verification).
        #[arg(long)]
        harness_checkout_root: PathBuf,
    },
    /// Validate a complete campaign NDJSON artifact and print its authoritative, self-contained JSON
    /// report to stdout (stdout is JSON only). This is the read-only stage-6 analysis path; it provisions
    /// and measures nothing.
    Analyze {
        /// Required `--campaign-root` directory shared with `Campaign`; `Analyze` is the sole reader of
        /// Campaign's fixed staged NDJSON under it and the sole writer of the final corrected-v2 bundle.
        #[arg(long)]
        campaign_root: PathBuf,
    },
    /// Run the whole preregistered campaign: every scheduled block and adjacent arm/control pair, in the
    /// seed's global order, provisioning a fresh isolated server per run and streaming one NDJSON
    /// observation per dose to the required output path. This is the full effectful measurement path.
    Campaign {
        /// Explicit `host:port` listen address every run's fresh isolated standalone binds to.
        #[arg(long)]
        server: String,
        /// Path to the built module WASM whose bytes are hash-verified before each publication.
        #[arg(long)]
        module_wasm: PathBuf,
        /// Explicit seed driving both the global block/arm-first order and every run's deterministic
        /// dataset — one seed, so the schedule and the seeded data cannot diverge.
        #[arg(long)]
        seed: u64,
        /// Required `--campaign-root` directory, never a caller-named output file. Campaign exclusively
        /// creates its one fixed non-authoritative staged NDJSON under `<campaign-root>/staging/`,
        /// failing if it already exists.
        #[arg(long)]
        campaign_root: PathBuf,
        /// Git worktree checked out against the harness's own embedded build commit. Phase 1:
        /// accepted and threaded but not yet compared (Phase 2: `BuildProvenance`'s
        /// `EmbeddedHarnessCommit` checkout verification, required clean, and per-run recheck).
        #[arg(long)]
        harness_checkout_root: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Plan => {
            let schedule = Schedule::preregistered();
            for &cell in schedule.cells() {
                println!(
                    "{cell:?}: regime={:?} predicted={:?} control={:?}",
                    cell.growth_regime(),
                    cell.predicted_response(),
                    cell.control_table(),
                );
            }
            Ok(())
        }
        Command::Provision {
            server,
            module_wasm,
            seed,
            cell_index,
            block,
            role,
            harness_checkout_root,
        } => {
            let listen = ListenAddress::parse(&server)?;
            let schedule = Schedule::preregistered();
            let block_run = schedule.canonical_block(cell_index, block)?;
            let run = RunCoordinate::new(&block_run, role);
            let manifest = provision_run(
                listen,
                &module_wasm,
                run,
                ScheduleSeed::new(seed),
                &harness_checkout_root,
            )?;
            println!("{}", serde_json::to_string_pretty(&manifest)?);
            Ok(())
        }
        Command::Run {
            server,
            module_wasm,
            seed,
            cell_index,
            block,
            role,
            harness_checkout_root,
        } => {
            let listen = ListenAddress::parse(&server)?;
            let schedule = Schedule::preregistered();
            let block_run = schedule.canonical_block(cell_index, block)?;
            let coordinate = RunCoordinate::new(&block_run, role);
            // The arm/control runs derive privately from the cell; select the requested one.
            let [arm, control] = block_run.cell().matched_runs();
            let run = match role {
                RunRole::Arm => arm,
                RunRole::Control => control,
            };
            provision_and_run(
                listen,
                &module_wasm,
                coordinate,
                ScheduleSeed::new(seed),
                &harness_checkout_root,
                |server, manifest| execute_run(server, manifest, run),
            )?;
            Ok(())
        }
        Command::Analyze { campaign_root } => {
            // Validate the fixed staged artifact under campaign_root into the trusted campaign graph and
            // durably publish the one authoritative report. Stdout carries JSON only.
            let root = CampaignRoot::new(campaign_root);
            let report = crate::analysis::analyze::analyze(&root)?;
            println!("{}", serde_json::to_string(&report)?);
            Ok(())
        }
        Command::Campaign {
            server,
            module_wasm,
            seed,
            campaign_root,
            harness_checkout_root,
        } => {
            let listen = ListenAddress::parse(&server)?;
            let root = CampaignRoot::new(campaign_root);
            let partial_path = CampaignPartialPath::under(&root);
            // Phase-1 skeleton ordering only, not the authoritative final contract: the spec requires
            // resolving harness provenance and rejecting a `Development` build *before* creating staging
            // output, which is deferred to Phase 2. Until that lands, this create-before-provisioning
            // order is temporary and must be revisited alongside the provenance rejection wiring.
            let sink = ObservationSink::create(&OutputPath::new(partial_path.path().to_path_buf()))
                .map_err(|e| {
                    anyhow::anyhow!("creating the campaign output sink: {}", e.diagnostic())
                })?;
            // Drive the whole campaign. A completed campaign reports its affine completion evidence; an
            // incomplete outcome or a pre-cleanup acquisition abort is a loud failure carrying its typed
            // evidence — the durable records already written remain on disk regardless.
            match run_campaign(
                listen,
                &module_wasm,
                ScheduleSeed::new(seed),
                sink,
                &harness_checkout_root,
            ) {
                Ok(CampaignOutcome::Complete(complete)) => {
                    println!("campaign complete: {complete:?}");
                    Ok(())
                }
                Ok(CampaignOutcome::Incomplete(incomplete)) => {
                    Err(anyhow::anyhow!("campaign incomplete: {incomplete:?}"))
                }
                Err(aborted) => Err(anyhow::anyhow!(
                    "campaign aborted during run acquisition: {aborted:?}"
                )),
            }
        }
    }
}
