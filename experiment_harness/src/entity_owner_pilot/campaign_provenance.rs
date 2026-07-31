//! The campaign-constant provenance every Pilot record is interpreted under.

use anyhow::{Context, Result};
use semver::Version;
use serde::Serialize;

use crate::entity_owner_pilot::harness_build_provenance::HarnessBuildProvenance;
use crate::entity_owner_pilot::pilot_parameters::PilotParameters;
use crate::manifest::build_provenance::BuildProvenance;
use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::params::{EXPECTED_RELEASE_COMMIT, EXPECTED_VERSION};

/// The facts that hold for the whole Pilot, recorded once with the frozen inventory.
///
/// These are the *expected* pins — what every attempt will be checked against — not observations:
/// they are knowable before any server exists, which is why this record survives even a provisioning
/// failure on the very first attempt. Each attempt's *actual* runtime and published instance are
/// recorded separately by
/// [`AttemptProvenance`](super::attempt_provenance::AttemptProvenance).
///
/// The candidate version is deliberately absent: every
/// [`AttemptKey`](super::attempt_key::AttemptKey) already carries it, and a second copy here could
/// disagree with the attempts it claims to describe.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PilotCampaignProvenance {
    harness: HarnessBuildProvenance,
    expected_version: Version,
    expected_release_commit: ReleaseCommit,
    expected_module_wasm_sha256: WasmSha256,
    parameters: PilotParameters,
}

impl PilotCampaignProvenance {
    /// Resolve the campaign's expected pins from the harness's own build evidence and preregistered
    /// constants.
    ///
    /// The version and release commit go through the same validated parsers
    /// ([`Version::parse`], [`ReleaseCommit::parse`]) that
    /// [`VerifiedCli`](crate::manifest::verified_cli::VerifiedCli) checks a real binary against, so
    /// the recorded pin is the same typed fact rather than a parallel unparsed string.
    pub(crate) fn resolved() -> Result<Self> {
        let expected_version = Version::parse(EXPECTED_VERSION)
            .with_context(|| format!("EXPECTED_VERSION is not semver: {EXPECTED_VERSION:?}"))?;
        let expected_release_commit = ReleaseCommit::parse(EXPECTED_RELEASE_COMMIT)
            .context("EXPECTED_RELEASE_COMMIT is not a canonical hex commit")?;
        Ok(Self {
            harness: HarnessBuildProvenance::of(BuildProvenance::from_build_env()),
            expected_version,
            expected_release_commit,
            expected_module_wasm_sha256: WasmSha256::new(MODULE_WASM_SHA256),
            parameters: PilotParameters::preregistered(),
        })
    }
}
