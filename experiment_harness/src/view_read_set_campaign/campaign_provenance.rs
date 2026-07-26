//! The campaign-constant provenance every record of this campaign is interpreted under.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because these are the pins
//! every attempt is admitted against, and their authority comes from having been resolved by
//! [`CampaignProvenance::resolved`] out of the harness's own build evidence and the preregistered
//! constants. A private field is visible to its declaring module **and every descendant**, so a
//! `#[cfg(test)] mod tests` child, or any child added later, could write the struct literal and
//! record pins no build ever produced — and then every attempt would be admitted against a forged
//! standard, since the reconciliation gate compares to exactly this value. `sealed` has no children,
//! so the sole constructor really is the only door.

mod sealed {
    use anyhow::{Context, Result};
    use semver::Version;
    use serde::Serialize;

    use crate::entity_owner_pilot::harness_build_provenance::HarnessBuildProvenance;
    use crate::manifest::build_provenance::BuildProvenance;
    use crate::manifest::release_commit::ReleaseCommit;
    use crate::manifest::wasm_sha256::WasmSha256;
    use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
    use crate::params::{EXPECTED_RELEASE_COMMIT, EXPECTED_VERSION};
    use crate::view_read_set_campaign::campaign_parameters::CampaignParameters;

    /// The facts that hold for the whole campaign, recorded once with the frozen inventory.
    ///
    /// These are the *expected* pins — what every attempt will be checked against — not observations:
    /// they are knowable before any server exists, which is why this record survives even a
    /// provisioning failure on the very first attempt. Each attempt's *actual* runtime and published
    /// instance are recorded separately by
    /// [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance).
    ///
    /// Every field is private to this childless module and [`Self::resolved`] is the only
    /// constructor, so the pins an attempt is admitted against are always the ones this harness build
    /// and this preregistration produced.
    ///
    /// The candidate version is deliberately absent: every
    /// [`AttemptKey`](crate::view_read_set_campaign::attempt_key::AttemptKey) already carries it, and
    /// a second copy here could disagree with the attempts it claims to describe.
    ///
    /// [`HarnessBuildProvenance`] is reused from the completed Pilot's module rather than duplicated.
    /// It projects [`BuildProvenance`], a fact about the *harness binary* rather than about either
    /// campaign's experiment ontology, and a second projection could drift from it. The coupling is
    /// real and worth stating plainly: a change to that enum changes both ledgers' shape, so it must
    /// be treated as a shared contract rather than as `entity_owner_pilot`'s private type.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct CampaignProvenance {
        harness: HarnessBuildProvenance,
        expected_version: Version,
        expected_release_commit: ReleaseCommit,
        expected_module_wasm_sha256: WasmSha256,
        parameters: CampaignParameters,
    }

    impl CampaignProvenance {
        /// Resolve the campaign's expected pins from the harness's own build evidence and
        /// preregistered constants.
        ///
        /// The version and release commit go through the same validated parsers
        /// ([`Version::parse`], [`ReleaseCommit::parse`]) that
        /// [`VerifiedCli`](crate::manifest::verified_cli::VerifiedCli) checks a real binary against,
        /// so the recorded pin is the same typed fact rather than a parallel unparsed string.
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
                parameters: CampaignParameters::preregistered(),
            })
        }

        /// The pinned runtime version every attempt's CLI and standalone binary must report.
        pub(crate) fn expected_version(&self) -> &Version {
            &self.expected_version
        }

        /// The pinned upstream release commit every attempt's CLI must report.
        pub(crate) fn expected_release_commit(&self) -> ReleaseCommit {
            self.expected_release_commit
        }

        /// The pinned module artifact digest every attempt must have published.
        pub(crate) fn expected_module_wasm_sha256(&self) -> WasmSha256 {
            self.expected_module_wasm_sha256
        }

        /// This campaign's preregistered parameters, read by the reconciliation gate that checks
        /// each attempt's recorded provenance against these pins.
        pub(crate) fn parameters(&self) -> &CampaignParameters {
            &self.parameters
        }
    }
}

pub(crate) use sealed::CampaignProvenance;
