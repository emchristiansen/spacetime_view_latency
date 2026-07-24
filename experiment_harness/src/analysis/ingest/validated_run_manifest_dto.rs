//! Untrusted mirror of
//! [`ValidatedRunManifest`](crate::manifest::validated_run_manifest::ValidatedRunManifest).

use serde::Deserialize;

use crate::analysis::ingest::distribution_facts_dto::DistributionFactsDto;
use crate::analysis::ingest::module_facts_dto::ModuleFactsDto;
use crate::analysis::ingest::preregistered_parameters_dto::PreregisteredParametersDto;
use crate::analysis::ingest::role_identities_dto::RoleIdentitiesDto;
use crate::analysis::ingest::run_coordinate_dto::RunCoordinateDto;
use crate::analysis::ingest::server_facts_dto::ServerFactsDto;

/// The wire form of one run's immutable manifest snapshot. It is a *serialized* value carrying the
/// same syntax as a live-proven [`ValidatedRunManifest`], not that trusted value: the name only
/// records that the bytes had this shape. `validate` folds a set of these into the trusted campaign
/// graph — proving the *campaign-stable* facts (release/version, distribution and module artifact
/// hash, preregistered parameters, seed) homogeneous while validating the *per-run* facts (server
/// process, data/keys dirs, listen address/URL, database identity, role identities) only for internal
/// consistency, shape, and reference binding — or fails loud. Homogeneity is field-specific, never
/// whole-manifest equality; role identities in particular are required per-run facts, never
/// campaign-homogeneous facts (spec: "Role identities are required per-run facts, never
/// campaign-homogeneous facts").
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ValidatedRunManifestDto {
    pub(crate) run: RunCoordinateDto,
    pub(crate) schedule_seed: u64,
    pub(crate) parameters: PreregisteredParametersDto,
    pub(crate) distribution: DistributionFactsDto,
    pub(crate) server: ServerFactsDto,
    pub(crate) module: ModuleFactsDto,
    pub(crate) role_identities: RoleIdentitiesDto,
}
