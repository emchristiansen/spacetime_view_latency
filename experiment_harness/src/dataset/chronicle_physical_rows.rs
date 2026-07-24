//! The physical per-table row counts of the two-table Chronicle family.

use serde::Serialize;

/// The physical `message_visibility` and `chronicle_message` row counts for a Chronicle-family
/// dataset. One logical pair is exactly one visibility row and one message row, so the two counts
/// are always equal. Both fields are private and the only constructor is
/// [`Self::from_logical_pairs`], which sets them from a single logical-pair total — so unequal
/// per-table counts are unrepresentable rather than merely discouraged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct ChroniclePhysicalRows {
    visibility_rows: u64,
    message_rows: u64,
}

impl ChroniclePhysicalRows {
    /// The physical per-table counts for `logical_pairs` logical pairs: one `message_visibility`
    /// row and one `chronicle_message` row each, so both counts equal `logical_pairs`.
    pub(crate) fn from_logical_pairs(logical_pairs: u64) -> Self {
        Self {
            visibility_rows: logical_pairs,
            message_rows: logical_pairs,
        }
    }

    /// The physical `message_visibility`-table row count.
    pub(crate) fn visibility_rows(&self) -> u64 {
        self.visibility_rows
    }

    /// The physical `chronicle_message`-table row count.
    pub(crate) fn message_rows(&self) -> u64 {
        self.message_rows
    }
}
