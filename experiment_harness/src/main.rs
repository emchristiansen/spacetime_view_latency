//! SpacetimeDB 2.6.1 view read-set experiment harness.
//!
//! Phase 2 (in progress): the typed plan model, the version-selection/assertion and
//! immutable run-manifest types, and the server-provisioning capabilities/driver
//! (start/publish/shutdown of the `/proc`-proven pinned standalone) are present but not yet
//! verified end-to-end. Seeding, subscription, measurement, and deterministic randomization
//! remain stubbed. Several typed model members are consumed only in later phases, so
//! unused-code is allowed crate-wide for the skeleton.
#![allow(dead_code)]

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
        /// Required output path for the durable NDJSON observation stream (records go to a file, never
        /// stdout); created exclusively, so a pre-existing path fails fast.
        #[arg(long)]
        output: PathBuf,
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
        } => {
            let listen = ListenAddress::parse(&server)?;
            let schedule = Schedule::preregistered();
            let block_run = schedule.canonical_block(cell_index, block)?;
            let run = RunCoordinate::new(&block_run, role);
            let manifest = provision_run(listen, &module_wasm, run, ScheduleSeed::new(seed))?;
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
                |server, manifest| execute_run(server, manifest, run),
            )?;
            Ok(())
        }
        Command::Campaign {
            server,
            module_wasm,
            seed,
            output,
        } => {
            let listen = ListenAddress::parse(&server)?;
            // Create the one required output file exclusively before any provisioning, so a bad path or a
            // pre-existing file fails fast rather than after standing up a server.
            let sink = ObservationSink::create(&OutputPath::new(output)).map_err(|e| {
                anyhow::anyhow!("creating the campaign output sink: {}", e.diagnostic())
            })?;
            // Drive the whole campaign. A completed campaign reports its affine completion evidence; an
            // incomplete outcome or a pre-cleanup acquisition abort is a loud failure carrying its typed
            // evidence — the durable records already written remain on disk regardless.
            match run_campaign(listen, &module_wasm, ScheduleSeed::new(seed), sink) {
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
