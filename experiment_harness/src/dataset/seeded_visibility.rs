//! Live `message_visibility` rows read back after seeding, for pair-uniqueness checking.

use std::collections::HashSet;

use anyhow::{ensure, Result};
#[rustfmt::skip]
#[allow(unused_imports)]
use crate::module_artifact::bindings::*;
use spacetimedb_sdk::Table;

/// The subscribable name of the visibility table. A cross-component contract that must match
/// the module's `#[table(accessor = message_visibility)]` and the generated client accessor,
/// so it is a named constant, never an inline literal.
const TABLE_MESSAGE_VISIBILITY: &str = "message_visibility";

/// The live `message_visibility` rows read back from the server after seeding, retained as a
/// `Vec` so duplicates are never collapsed before detection.
///
/// The spec requires the dataset to "enforce and assert uniqueness of `(viewer,
/// message_uuid)` visibility pairs", and the Implementation-Time Decision requires the
/// harness to "assert pair uniqueness after seeding as defense in depth". This checks *actual
/// server state* — the rows the reducers wrote — not the plan: the plan's keys are disjoint
/// ranges and so trivially unique, so asserting the plan alone would certify nothing about
/// what the server actually stored. The module's insertion reducer already fails fast on a
/// duplicate `(viewer, message_uuid)`; reading the rows back and re-checking here is the
/// independent second line of defense.
pub(crate) struct SeededVisibility {
    rows: Vec<MessageVisibility>,
}

impl SeededVisibility {
    /// The `SELECT * FROM message_visibility` subscription that materializes every seeded
    /// visibility row (both the measured and the growth slice) into the client cache.
    pub(crate) fn subscription_sql() -> String {
        format!("SELECT * FROM {}", TABLE_MESSAGE_VISIBILITY)
    }

    /// Read every currently-subscribed `message_visibility` row out of the client cache,
    /// preserving duplicates.
    pub(crate) fn read_back(conn: &DbConnection) -> Self {
        Self {
            rows: conn.db.message_visibility().iter().collect(),
        }
    }

    /// Assert the seeded visibility rows are exactly `expected_pairs` distinct `(viewer,
    /// message_uuid)` pairs. Fails fast on either a cardinality mismatch or a duplicate pair.
    ///
    /// Duplicates are detected without being collapsed first: the read-back keeps every row,
    /// and a duplicate is caught by comparing the distinct-pair count to the row count, so a
    /// pair the reducer guard somehow admitted would still fail here. `viewer` is compared by
    /// canonical hex (matching the rest of the harness) so the check does not depend on
    /// `Identity` implementing `Hash`.
    pub(crate) fn assert_unique_pairs(&self, expected_pairs: u64) -> Result<()> {
        ensure!(
            self.rows.len() as u64 == expected_pairs,
            "seeded message_visibility cardinality mismatch: read back {} rows, expected {} \
             logical pairs",
            self.rows.len(),
            expected_pairs
        );
        let distinct: HashSet<(String, u64)> = self
            .rows
            .iter()
            .map(|row| (row.viewer.to_hex().to_string(), row.message_uuid))
            .collect();
        ensure!(
            distinct.len() == self.rows.len(),
            "duplicate (viewer, message_uuid) visibility pair in server state: {} distinct of \
             {} rows read back",
            distinct.len(),
            self.rows.len()
        );
        Ok(())
    }
}
