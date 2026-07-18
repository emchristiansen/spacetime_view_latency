//! Untrusted mirror of [`ManifestReference`](crate::observation::manifest_reference::ManifestReference).

use serde::Deserialize;

use crate::analysis::ingest::run_coordinate_dto::RunCoordinateDto;

/// The wire form of an observation's reference back to its run manifest: the manifest's identity
/// tuple of run coordinate, canonical-hex database identity, and transparent schedule seed. The hex
/// identity is carried verbatim as a string; `validate` parses it and proves each observation's
/// reference resolves bijectively to exactly one manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManifestReferenceDto {
    pub(crate) run: RunCoordinateDto,
    pub(crate) database_identity: String,
    pub(crate) schedule_seed: u64,
}
