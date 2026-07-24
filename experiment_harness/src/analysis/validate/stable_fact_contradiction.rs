//! A single campaign-stable fact's typed contradiction: expected and observed values of *one* fact.

use std::path::PathBuf;

use semver::Version;

use crate::analysis::validate::campaign_stable_fact::CampaignStableFact;
use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::wasm_sha256::WasmSha256;

/// A disagreement over one campaign-stable fact, each variant carrying that fact's `expected` and
/// `observed` values **in the fact's own domain type** and **of the same type on both sides**. Because
/// each variant fixes both sides to one fact, a cross-fact pairing (an expected [`Version`] against an
/// observed `batch_size`, say) is structurally unrepresentable — the invariant this milestone
/// establishes, not merely one maintained by discipline at construction sites.
///
/// This one enum serves both stable-fact contradictions:
/// - inside [`MalformedStableFault::OffSpecification`](super::malformed_stable_fault::MalformedStableFault),
///   `expected` is the frozen preregistered constant and `observed` is the parsed manifest value;
/// - inside [`CampaignFactHeterogeneity`](super::integrity_error::IntegrityError), `expected` is the
///   reference run's value and `observed` is the diverging run's value (the reference run coordinate is
///   carried by the enclosing variant).
///
/// The two `…VersionRaw` fields are the manifest's *raw* unparsed version strings, whose own domain is
/// `String`; every other variant is a parsed domain type or a native numeric/boolean parameter.
#[derive(Debug)]
pub(crate) enum StableFactContradiction {
    NixStoreBinDir { expected: PathBuf, observed: PathBuf },
    CliExe { expected: PathBuf, observed: PathBuf },
    CliVersion { expected: Version, observed: Version },
    CliReleaseCommit {
        expected: ReleaseCommit,
        observed: ReleaseCommit,
    },
    CliVersionRaw { expected: String, observed: String },
    StandaloneExe { expected: PathBuf, observed: PathBuf },
    StandaloneVersion { expected: Version, observed: Version },
    StandaloneVersionRaw { expected: String, observed: String },
    WasmSha256 {
        expected: WasmSha256,
        observed: WasmSha256,
    },
    ScheduleSeed {
        expected: ScheduleSeed,
        observed: ScheduleSeed,
    },
    BatchSize { expected: u64, observed: u64 },
    NumDoses { expected: u64, observed: u64 },
    DoseLadder {
        expected: Vec<u64>,
        observed: Vec<u64>,
    },
    BatchDelayMs { expected: u64, observed: u64 },
    RepetitionBlocks { expected: u32, observed: u32 },
    ConfirmedReads { expected: bool, observed: bool },
}

impl StableFactContradiction {
    /// The closed discriminator naming which stable fact this contradiction concerns.
    pub(crate) fn fact(&self) -> CampaignStableFact {
        match self {
            Self::NixStoreBinDir { .. } => CampaignStableFact::NixStoreBinDir,
            Self::CliExe { .. } => CampaignStableFact::CliExe,
            Self::CliVersion { .. } => CampaignStableFact::CliVersion,
            Self::CliReleaseCommit { .. } => CampaignStableFact::CliReleaseCommit,
            Self::CliVersionRaw { .. } => CampaignStableFact::CliVersionRaw,
            Self::StandaloneExe { .. } => CampaignStableFact::StandaloneExe,
            Self::StandaloneVersion { .. } => CampaignStableFact::StandaloneVersion,
            Self::StandaloneVersionRaw { .. } => CampaignStableFact::StandaloneVersionRaw,
            Self::WasmSha256 { .. } => CampaignStableFact::WasmSha256,
            Self::ScheduleSeed { .. } => CampaignStableFact::ScheduleSeed,
            Self::BatchSize { .. } => CampaignStableFact::BatchSize,
            Self::NumDoses { .. } => CampaignStableFact::NumDoses,
            Self::DoseLadder { .. } => CampaignStableFact::DoseLadder,
            Self::BatchDelayMs { .. } => CampaignStableFact::BatchDelayMs,
            Self::RepetitionBlocks { .. } => CampaignStableFact::RepetitionBlocks,
            Self::ConfirmedReads { .. } => CampaignStableFact::ConfirmedReads,
        }
    }
}
