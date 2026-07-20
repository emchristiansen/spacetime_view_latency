//! Provenance-retention proof: the campaign root retains both the parsed and the raw version, as a pair.
//!
//! The fold parses each self-reported version string into a `semver::Version` for the off-specification
//! comparison, but retains the verbatim raw string *alongside* the parsed value rather than in place of it,
//! so the report can show both what the run reported and how it parsed. This proof asserts both
//! representations survive on the campaign root for the CLI and the standalone.

use semver::Version;

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::params::EXPECTED_VERSION;

/// The complete campaign folds, and its root `CampaignProvenance` carries, for both binaries, the parsed
/// `semver::Version` (with its structured major/minor/patch components) and the raw self-reported version
/// string verbatim — the two are retained as a distinct-typed pair, not collapsed to one representation.
#[test]
fn the_campaign_provenance_retains_both_parsed_and_raw_versions() {
    let campaign = ValidatedCampaign::from_records(CampaignFixture::valid().into_records())
        .expect("the complete, spec-correct campaign folds without an integrity error");
    let provenance = campaign.provenance();

    let expected_parsed =
        Version::parse(EXPECTED_VERSION).expect("EXPECTED_VERSION is a valid semver constant");

    // The parsed side is the structured semver value, not a re-wrapped string.
    assert_eq!(
        provenance.cli_version(),
        &expected_parsed,
        "the root retains the parsed CLI semver version"
    );
    assert_eq!(
        provenance.standalone_version(),
        &expected_parsed,
        "the root retains the parsed standalone semver version"
    );
    assert_eq!(
        (
            provenance.cli_version().major,
            provenance.cli_version().minor,
            provenance.cli_version().patch,
        ),
        (2, 6, 1),
        "the parsed CLI version exposes its structured semver components, proving it is not a raw string"
    );

    // The raw side is the verbatim self-reported text, retained alongside the parsed value.
    assert_eq!(
        provenance.cli_version_raw(),
        EXPECTED_VERSION,
        "the root retains the raw self-reported CLI version string verbatim"
    );
    assert_eq!(
        provenance.standalone_version_raw(),
        EXPECTED_VERSION,
        "the root retains the raw self-reported standalone version string verbatim"
    );
}
