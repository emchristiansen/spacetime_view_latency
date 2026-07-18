//! Untrusted mirror of the sink's private `ObservationBody`
//! (`crate::observation::observation_sink`).

use serde::Deserialize;

use crate::analysis::ingest::dose_observation_dto::DoseObservationDto;

/// The wire form of a dose observation record line's body: the single wrapped observation. The wrapper
/// exists so the manifest and observation line bodies are distinct object shapes even before their
/// record kind is consulted.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObservationBodyDto {
    pub(crate) observation: DoseObservationDto,
}
