//! The physical row count of the single-table `message` family.

use serde::Serialize;

/// The physical `message`-table row count for a `message`-family dataset, where one logical row is
/// exactly one physical row. The field is private and the only constructor is
/// [`Self::from_logical_rows`], so a count is always a faithful projection of a logical row total.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct MessagePhysicalRows {
    message_rows: u64,
}

impl MessagePhysicalRows {
    /// The physical count for `logical_rows` logical rows — identical, since the family is a single
    /// table with one physical row per logical row.
    pub(crate) fn from_logical_rows(logical_rows: u64) -> Self {
        Self {
            message_rows: logical_rows,
        }
    }

    /// The physical `message`-table row count.
    pub(crate) fn message_rows(&self) -> u64 {
        self.message_rows
    }
}
