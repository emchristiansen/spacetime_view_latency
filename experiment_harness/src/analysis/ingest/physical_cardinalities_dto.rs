//! Untrusted mirror of [`PhysicalCardinalities`](crate::dataset::physical_cardinalities::PhysicalCardinalities).

use serde::Deserialize;

use crate::analysis::ingest::chronicle_physical_rows_dto::ChroniclePhysicalRowsDto;
use crate::analysis::ingest::message_physical_rows_dto::MessagePhysicalRowsDto;

/// The wire form of a dose's physical table cardinalities: an externally-tagged newtype variant per
/// family. The closed variant set mirrors the trusted enum, so a family carrying the wrong table's
/// count shape is unrepresentable *at the wire level* (there is no zero placeholder for an unused
/// table); an unknown family tag is rejected by serde. What `validate` proves is semantic: that the
/// family matches the run's arm/control table and the counts equal the preregistered cardinalities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum PhysicalCardinalitiesDto {
    Message(MessagePhysicalRowsDto),
    Chronicle(ChroniclePhysicalRowsDto),
}
