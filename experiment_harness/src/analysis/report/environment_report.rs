//! The campaign-homogeneous environment/provenance projection at the report root.

use std::path::PathBuf;

use serde::Serialize;

use crate::analysis::report::preregistered_parameters_report::PreregisteredParametersReport;
use crate::analysis::validate::validated_campaign::ValidatedCampaign;

/// The campaign-stable environment projection (spec: "`ValidatedCampaign` owns the campaign-homogeneous
/// distribution/module facts ... schedule seed, and preregistered parameters"). Projected once at the root
/// from the campaign's [`CampaignProvenance`](crate::analysis::validate::campaign_provenance::CampaignProvenance),
/// schedule seed, and preregistered parameters — the values validation proved identical across every run.
///
/// The typed trusted domain values cross the lossy boundary here: paths become [`PathBuf`] (serialized as
/// their string form), versions and canonical-hex hashes become their `String` renderings, and the seed
/// becomes a bare `u64`. All are exact — no float is involved, so no [`FiniteF64`](crate::analysis::finite_f64::FiniteF64)
/// is needed at this projection.
#[derive(Debug, Serialize)]
pub(crate) struct EnvironmentReport {
    /// The pinned Nix-store `bin` directory both binaries resolve from.
    nix_store_bin_dir: PathBuf,
    /// The verified CLI executable path.
    cli_exe: PathBuf,
    /// The parsed CLI semver version, rendered canonically.
    cli_version: String,
    /// The CLI's canonical lowercase-hex release commit.
    cli_release_commit: String,
    /// The CLI's raw self-reported version string, verbatim.
    cli_version_raw: String,
    /// The verified standalone executable path.
    standalone_exe: PathBuf,
    /// The parsed standalone semver version, rendered canonically.
    standalone_version: String,
    /// The standalone's raw self-reported version string, verbatim.
    standalone_version_raw: String,
    /// The `/proc/<pid>/exe` resolved standalone executable, proven campaign-stable.
    resolved_exe: PathBuf,
    /// The published module's canonical-hex WASM SHA-256 artifact hash.
    wasm_sha256: String,
    /// The one campaign-stable schedule seed.
    schedule_seed: u64,
    /// The preregistered parameters, proven identical to the frozen values across every run.
    parameters: PreregisteredParametersReport,
}

impl EnvironmentReport {
    /// Project the campaign's homogeneous environment. Takes the whole `&ValidatedCampaign` because the
    /// seed and parameters are owned by the campaign root while the distribution/module facts are owned by
    /// its [`CampaignProvenance`](crate::analysis::validate::campaign_provenance::CampaignProvenance) — one
    /// input, projected in one place.
    pub(crate) fn of(campaign: &ValidatedCampaign) -> Self {
        let _ = campaign;
        todo!("Phase 2: project provenance paths/versions/hashes, schedule seed, and parameters")
    }
}
