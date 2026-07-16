//! SpacetimeDB 2.6.1 view read-set experiment harness.
//!
//! Phase 1 skeleton: the typed plan model is complete and compiles; provisioning,
//! seeding, subscription, measurement, and randomization are stubbed. Several typed
//! model members (roles, run roles, manifest fields) are consumed only in later
//! phases, so unused-code is allowed crate-wide for the skeleton.
#![allow(dead_code)]

mod execute_run;
mod params;
mod plan;
mod provision;
mod roles;
mod run_manifest;

use anyhow::Result;
use clap::Parser;

use crate::plan::schedule::Schedule;

/// Command-line interface.
#[derive(Parser)]
#[command(about = "SpacetimeDB 2.6.1 view read-set experiment harness")]
struct Cli {
    /// Explicit seed for deterministic global randomization of block order.
    #[arg(long)]
    seed: u64,
    /// Print the preregistered plan and exit, without provisioning or measuring.
    #[arg(long)]
    plan_only: bool,
    /// Listen address for the isolated experiment standalone server. Required: no
    /// default listen address is assumed (fail-fast, no silent fallback).
    #[arg(long)]
    server: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let schedule = Schedule::preregistered();

    if cli.plan_only {
        for &cell in schedule.cells() {
            println!(
                "{cell:?}: regime={:?} predicted={:?} control={:?}",
                cell.growth_regime(),
                cell.predicted_response(),
                cell.control_table(),
            );
        }
        return Ok(());
    }

    let manifest = provision::provision(&cli.server, cli.seed)?;
    for block in schedule.randomized_block_order(cli.seed) {
        for run in block.cell().matched_runs() {
            execute_run::execute_run(&manifest, run)?;
        }
    }
    Ok(())
}
