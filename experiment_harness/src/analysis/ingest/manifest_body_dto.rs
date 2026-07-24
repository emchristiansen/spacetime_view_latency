//! Untrusted mirror of the sink's private `ManifestBody`
//! (`crate::observation::observation_sink`).

use serde::Deserialize;

use crate::analysis::ingest::manifest_reference_dto::ManifestReferenceDto;
use crate::analysis::ingest::validated_run_manifest_dto::ValidatedRunManifestDto;

/// The wire form of a manifest record line's body: the sink-derived reference plus the immutable
/// manifest. The reference and the manifest independently carry a run coordinate and identity;
/// `validate` proves they agree, so a body whose reference disagrees with its own manifest is rejected
/// rather than silently resolved to either.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManifestBodyDto {
    pub(crate) reference: ManifestReferenceDto,
    pub(crate) manifest: ValidatedRunManifestDto,
}
