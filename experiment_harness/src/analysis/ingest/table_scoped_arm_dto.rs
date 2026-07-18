//! Untrusted mirror of [`TableScopedArm`](crate::plan::table_scoped_arm::TableScopedArm).

use serde::Deserialize;

/// The wire form of a table-scoped arm (A–E), the externally-tagged unit variant string. An unknown
/// arm string is rejected by serde as it deserializes, closing the tag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum TableScopedArmDto {
    ProceduralRange,
    QueryFull,
    QuerySemijoin,
    QueryFullPk,
    QuerySemijoinPk,
}
