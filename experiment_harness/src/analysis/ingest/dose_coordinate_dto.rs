//! Untrusted mirror of [`DoseCoordinate`](crate::observation::dose_coordinate::DoseCoordinate).

use serde::Deserialize;

use crate::analysis::ingest::run_coordinate_dto::RunCoordinateDto;

/// The wire form of a dose observation's ladder coordinate: the run it belongs to, its 1-based dose
/// index, the driving role's canonical tag, and the cumulative logical x-axis count. The dose and
/// x-axis are bare numbers and the tag a bare string; `validate` proves the dose agrees with the
/// enclosing record kind, the tag matches the run's driving role, and `logical_n` equals the
/// preregistered `dose * BATCH_SIZE`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DoseCoordinateDto {
    pub(crate) run: RunCoordinateDto,
    pub(crate) dose: u64,
    pub(crate) driving_role_tag: String,
    pub(crate) logical_n: u64,
}
