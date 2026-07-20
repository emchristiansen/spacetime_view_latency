//! The per-run provenance projection embedded in each run report.

use std::path::PathBuf;

use serde::Serialize;

use crate::analysis::validate::run_provenance::RunProvenance;

/// One run's run-varying server/module provenance, projected into its run report (spec: "run-varying
/// values remain attached to each arm/control run"). Projected from the trusted
/// [`RunProvenance`](crate::analysis::validate::run_provenance::RunProvenance): the database identity and
/// listen address become their `String` renderings, the PID a bare `u32`, and the data/keys directories
/// [`PathBuf`]s. All exact — no float, so no [`FiniteF64`](crate::analysis::finite_f64::FiniteF64).
#[derive(Debug, Serialize)]
pub(crate) struct RunProvenanceReport {
    /// The run's published database identity, canonical-hex.
    database_identity: String,
    /// The run's server process id.
    pid: u32,
    /// The run's parsed listen socket address (host:port), rendered.
    listen_addr: String,
    /// The run's client URL, the `http://{listen_addr}` derivation.
    client_url: String,
    /// The run's fresh data directory.
    data_dir: PathBuf,
    /// The run's ephemeral JWT keys directory.
    keys_dir: PathBuf,
}

impl RunProvenanceReport {
    /// Project one run's trusted run-varying provenance into its report shape.
    pub(crate) fn of(provenance: &RunProvenance) -> Self {
        let _ = provenance;
        todo!("Phase 2: project the run's database identity, PID, addresses, and directories")
    }
}
