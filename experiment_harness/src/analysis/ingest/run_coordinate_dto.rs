//! Untrusted mirror of [`RunCoordinate`](crate::manifest::run_coordinate::RunCoordinate).

use serde::Deserialize;

use crate::analysis::ingest::cell_dto::CellDto;
use crate::analysis::ingest::run_role_dto::RunRoleDto;

/// The wire form of a run's identity coordinate: its cell, its arm/control role, and its 0-based
/// repetition-block index. The block index is not range-checked here; `validate` proves it is one of
/// the fixed `0..REPETITION_BLOCKS` positions and that each `(cell, role, block)` appears exactly once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunCoordinateDto {
    pub(crate) cell: CellDto,
    pub(crate) role: RunRoleDto,
    pub(crate) repetition_block: u32,
}
