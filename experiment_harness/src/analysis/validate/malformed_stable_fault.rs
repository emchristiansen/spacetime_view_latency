//! The typed contradiction behind a
//! [`MalformedStableProvenance`](super::integrity_error::IntegrityError) failure.

use crate::analysis::validate::stable_fact_contradiction::StableFactContradiction;
use crate::manifest::release_commit_parse_error::ReleaseCommitParseError;
use crate::manifest::wasm_sha256_parse_error::WasmSha256ParseError;

/// Why a campaign-stable provenance or parameter value is itself malformed or off the preregistered
/// specification — independent of cross-run homogeneity, which is a separate
/// [`CampaignFactHeterogeneity`](super::integrity_error::IntegrityError) failure.
///
/// Every way a raw wire value fails to become its domain value is its **own fact-specific variant**
/// carrying the exact typed parse error — never a bare `raw`-plus-prose reason. `raw` is retained
/// alongside the typed error only because parsing that string into the domain type is precisely what
/// failed, so no domain value can be formed; the *reason* it failed is the typed `error`, which locates
/// the defect (bad semver, wrong length, non-hex, non-canonical case) rather than describing it in text.
/// The fact-specific split makes a cross-fact pairing (a release-commit error attributed to the module
/// hash, say) structurally unrepresentable.
#[derive(Debug)]
pub(crate) enum MalformedStableFault {
    /// The campaign-stable CLI version string did not parse as semver. `raw` is the wire string that
    /// failed and `error` is the typed [`semver::Error`] naming why. The fact is fixed by the variant:
    /// only [`CliVersion`](super::campaign_stable_fact::CampaignStableFact::CliVersion) parses this way —
    /// [`CliVersionRaw`](super::campaign_stable_fact::CampaignStableFact::CliVersionRaw) has domain
    /// `String` and is never unparseable.
    UnparseableCliVersion {
        raw: String,
        error: semver::Error,
    },
    /// The campaign-stable standalone version string did not parse as semver. `raw` is the wire string
    /// that failed and `error` is the typed [`semver::Error`] naming why. The fact is fixed by the
    /// variant: only
    /// [`StandaloneVersion`](super::campaign_stable_fact::CampaignStableFact::StandaloneVersion) parses
    /// this way — [`StandaloneVersionRaw`](super::campaign_stable_fact::CampaignStableFact::StandaloneVersionRaw)
    /// has domain `String` and is never unparseable.
    UnparseableStandaloneVersion {
        raw: String,
        error: semver::Error,
    },
    /// The campaign-stable CLI release commit did not parse as a canonical lowercase-hex commit. `raw` is
    /// the wire string that failed and `error` is the typed [`ReleaseCommitParseError`] locating the
    /// defect. The fact is fixed by the variant: only
    /// [`CliReleaseCommit`](super::campaign_stable_fact::CampaignStableFact::CliReleaseCommit) parses this
    /// way.
    UnparseableReleaseCommit {
        raw: String,
        error: ReleaseCommitParseError,
    },
    /// The campaign-stable module WASM digest did not parse as a canonical lowercase-hex SHA-256. `raw` is
    /// the wire string that failed and `error` is the typed [`WasmSha256ParseError`] locating the defect.
    /// The fact is fixed by the variant: only
    /// [`WasmSha256`](super::campaign_stable_fact::CampaignStableFact::WasmSha256) parses this way.
    UnparseableWasmSha256 {
        raw: String,
        error: WasmSha256ParseError,
    },
    /// The value parsed into its domain type but is off the frozen preregistered specification (wrong
    /// version/commit, a parameter not equal to its preregistered constant). The
    /// [`StableFactContradiction`] fixes both sides to one fact in one domain type, so `expected` (the
    /// frozen constant) and `observed` (the parsed value) can never be a cross-fact pairing.
    OffSpecification(StableFactContradiction),
}
