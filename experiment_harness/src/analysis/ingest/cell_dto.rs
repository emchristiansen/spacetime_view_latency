//! Untrusted mirror of [`Cell`](crate::plan::cell::Cell).

use serde::Deserialize;

use crate::analysis::ingest::key_scoped_arm_dto::KeyScopedArmDto;
use crate::analysis::ingest::table_scoped_arm_dto::TableScopedArmDto;

/// The wire form of a `(arm, growth-regime)` cell: an externally-tagged newtype variant whose tag is
/// the regime family and whose payload is the arm. An unknown regime tag or arm string is rejected by
/// serde as it deserializes.
///
/// This variant set is the exact mirror of the trusted [`Cell`](crate::plan::cell::Cell)'s, so it
/// structurally excludes a table-scoped arm under own-slice growth *at the wire level* — there is no
/// `TableScopedOwnSlice` variant to deserialize into, exactly as production has no such state. What
/// remains for the `validate` pass is not this representability guarantee but semantic cross-record
/// validity: that every one of the nine cells is present exactly once with its full matched
/// arm/control block set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum CellDto {
    TableScopedUnrelated(TableScopedArmDto),
    KeyScopedUnrelated(KeyScopedArmDto),
    KeyScopedOwnSlice(KeyScopedArmDto),
}
