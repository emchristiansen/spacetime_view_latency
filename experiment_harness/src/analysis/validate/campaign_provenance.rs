//! The campaign-homogeneous manifest provenance retained on the trusted graph root.

use std::path::{Path, PathBuf};

use semver::Version;

use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::manifest::wasm_sha256::WasmSha256;

/// The campaign-stable environment/distribution/module provenance that validation proves *homogeneous*
/// across every run's manifest (spec: "`ValidatedCampaign` owns the campaign-homogeneous
/// distribution/module facts (actual CLI and standalone versions, release commit, WASM hash, Nix-store
/// executable paths, resolved standalone executable ...)"). Every run resolves the one pinned Nix-store
/// distribution and publishes the one verified module, so these facts are proven identical campaign-wide
/// and retained once at the root — not the schedule seed and preregistered parameters, which the
/// [`ValidatedCampaign`] already owns as its own fields.
///
/// [`ValidatedCampaign`]: super::validated_campaign::ValidatedCampaign
///
/// Every value is the *already-parsed typed* domain value validation produced, not a re-stringified
/// mirror: filesystem paths are [`PathBuf`], versions are [`semver::Version`], the release commit and WASM
/// digest are their canonical-hex domain newtypes ([`ReleaseCommit`]/[`WasmSha256`]). The two `_raw`
/// version strings are retained verbatim because they *are* the raw self-reported text — the parsed
/// [`Version`] is stored alongside, not in place of them. Projected from the typed trusted
/// [`ValidatedRunManifest`]'s distribution/module facts — the object already bound to the run after
/// manifest validation — never from the untrusted pre-validation DTO's strings.
///
/// Fields are private with no defaults; [`Self::mint`] is `pub(super)`, so a `CampaignProvenance` is
/// assembled only from within the `validate` subtree — in production only by the single validation pass
/// once it has proven campaign-wide homogeneity. This is trusted-graph content, not a report DTO: it is
/// not `Serialize`; the report projects it into its own
/// [`EnvironmentReport`](crate::analysis::report::environment_report::EnvironmentReport).
#[derive(Debug, Clone)]
pub(crate) struct CampaignProvenance {
    /// The pinned Nix-store `bin` directory both binaries resolve from.
    nix_store_bin_dir: PathBuf,
    /// The verified CLI executable path.
    cli_exe: PathBuf,
    /// The parsed, expected-matched CLI semver version.
    cli_version: Version,
    /// The CLI's canonical lowercase-hex release commit.
    cli_release_commit: ReleaseCommit,
    /// The CLI's raw self-reported version string, retained verbatim.
    cli_version_raw: String,
    /// The verified standalone executable path.
    standalone_exe: PathBuf,
    /// The parsed, expected-matched standalone semver version.
    standalone_version: Version,
    /// The standalone's raw self-reported version string, retained verbatim.
    standalone_version_raw: String,
    /// The `/proc/<pid>/exe` resolved standalone executable, proven equal to [`Self::standalone_exe`]
    /// across the campaign — a campaign-stable server fact.
    resolved_exe: PathBuf,
    /// The published module's canonical-hex WASM SHA-256 artifact hash.
    wasm_sha256: WasmSha256,
}

impl CampaignProvenance {
    /// Mint the campaign-homogeneous provenance from the typed trusted [`ValidatedRunManifest`] already
    /// proven representative of the homogeneous campaign, reusing the domain values it already carries
    /// rather than reparsing pre-validation DTO strings. `pub(super)` so only the `validate` subtree's
    /// pass — which owns proving that every run's manifest agrees on these facts — can construct one.
    pub(super) fn mint(manifest: &ValidatedRunManifest) -> Self {
        let _ = manifest;
        todo!("Phase 2: project the proven-homogeneous distribution/module facts into typed domain values")
    }

    /// The pinned Nix-store `bin` directory.
    pub(crate) fn nix_store_bin_dir(&self) -> &Path {
        &self.nix_store_bin_dir
    }

    /// The verified CLI executable path.
    pub(crate) fn cli_exe(&self) -> &Path {
        &self.cli_exe
    }

    /// The parsed CLI semver version.
    pub(crate) fn cli_version(&self) -> &Version {
        &self.cli_version
    }

    /// The CLI's canonical-hex release commit.
    pub(crate) fn cli_release_commit(&self) -> ReleaseCommit {
        self.cli_release_commit
    }

    /// The CLI's raw self-reported version string.
    pub(crate) fn cli_version_raw(&self) -> &str {
        &self.cli_version_raw
    }

    /// The verified standalone executable path.
    pub(crate) fn standalone_exe(&self) -> &Path {
        &self.standalone_exe
    }

    /// The parsed standalone semver version.
    pub(crate) fn standalone_version(&self) -> &Version {
        &self.standalone_version
    }

    /// The standalone's raw self-reported version string.
    pub(crate) fn standalone_version_raw(&self) -> &str {
        &self.standalone_version_raw
    }

    /// The `/proc/<pid>/exe` resolved standalone executable.
    pub(crate) fn resolved_exe(&self) -> &Path {
        &self.resolved_exe
    }

    /// The published module's WASM SHA-256 artifact hash.
    pub(crate) fn wasm_sha256(&self) -> WasmSha256 {
        self.wasm_sha256
    }
}
