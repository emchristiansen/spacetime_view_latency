//! Untrusted mirror of [`DoseObservation`](crate::observation::dose_observation::DoseObservation).

use serde::Deserialize;

use crate::analysis::ingest::dose_coordinate_dto::DoseCoordinateDto;
use crate::analysis::ingest::event_evidence_dto::EventEvidenceDto;
use crate::analysis::ingest::latency_summary_dto::LatencySummaryDto;
use crate::analysis::ingest::manifest_reference_dto::ManifestReferenceDto;
use crate::analysis::ingest::physical_cardinalities_dto::PhysicalCardinalitiesDto;

/// The wire form of one cumulative dose's complete observation: the manifest reference, ladder
/// coordinate, physical cardinalities, the lossless raw latency vector, its median/IQR summary, and
/// the SDK event evidence. The raw latencies are the flat nanosecond array exactly as serialized; the
/// fixed `BATCH_SIZE` count is *not* encoded by `Vec<u128>`, so `validate` proves the count and
/// recomputes the summary from this vector. Preserving every field verbatim is what lets `validate`
/// recompute — not merely re-parse — every reported median, IQR, cardinality, and event identity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DoseObservationDto {
    pub(crate) manifest_ref: ManifestReferenceDto,
    pub(crate) coordinate: DoseCoordinateDto,
    pub(crate) cardinalities: PhysicalCardinalitiesDto,
    pub(crate) latencies: Vec<u128>,
    pub(crate) summary: LatencySummaryDto,
    pub(crate) events: EventEvidenceDto,
}
