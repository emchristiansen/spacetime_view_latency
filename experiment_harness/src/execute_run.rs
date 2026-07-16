//! Execute one arm-or-control run of the dose ladder.

use anyhow::Result;

use crate::plan::run::Run;
use crate::run_manifest::RunManifest;

/// Execute one [`Run`]: seed the dataset for its cell's growth regime, subscribe to
/// the arm view or the matched direct control table, drive the preregistered dose
/// ladder with confirmed round trips, run correctness checks outside the measured
/// interval, and emit raw per-observation records. Stubbed in Phase 1.
pub fn execute_run(_manifest: &RunManifest, _run: Run) -> Result<()> {
    todo!("seed, subscribe, drive doses, confirm round trips, correctness-check, emit records")
}
