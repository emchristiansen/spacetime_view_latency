//! SpacetimeDB 2.6.1 view read-set experiment harness.
//!
//! Phase 2 (in progress): the typed plan model, the version-selection/assertion and
//! immutable run-manifest types, and the server-provisioning capabilities/driver
//! (start/publish/shutdown of the `/proc`-proven pinned standalone) are present but not yet
//! verified end-to-end. Seeding, subscription, measurement, and deterministic randomization
//! remain stubbed. Several typed model members are consumed only in later phases, so
//! unused-code is allowed crate-wide for the skeleton.
#![allow(dead_code)]

mod execute_run;
mod manifest;
mod module_artifact;
mod params;
mod plan;
mod provision;
mod roles;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::manifest::listen_address::ListenAddress;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::run_role::RunRole;
use crate::plan::schedule::Schedule;
use crate::provision::provision::provision_run;

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
    }
}
