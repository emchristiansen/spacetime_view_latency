//! Execute one arm-or-control run of the dose ladder.

use anyhow::Result;

use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::plan::run::Run;
use crate::provision::running_pinned_server::RunningPinnedServer;

/// Execute one [`Run`] against the provisioned `server`, recording under `manifest`: seed the
/// dataset for its cell's growth regime, subscribe to the arm view or the matched direct
/// control table, drive the preregistered dose ladder with confirmed round trips, run
/// correctness checks outside the measured interval, and emit raw per-observation records.
/// Stubbed until the measurement phase.
pub(crate) fn execute_run(
    _server: &RunningPinnedServer,
    _manifest: &ValidatedRunManifest,
    _run: Run,
) -> Result<()> {
    todo!("seed, subscribe, drive doses, confirm round trips, correctness-check, emit records")
}
