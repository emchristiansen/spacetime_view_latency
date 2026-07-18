//! Untrusted mirror of
//! [`ChroniclePhysicalRows`](crate::dataset::chronicle_physical_rows::ChroniclePhysicalRows).

use serde::Deserialize;

/// The wire form of the two-table Chronicle family's physical row counts. The trusted type keeps the
/// two counts equal by construction from a single logical-pair total, but the wire carries two
/// independent numbers, so their equality is *not* encoded here — `validate` proves the visibility and
/// message counts are equal and both equal the preregistered per-pair cardinality for the run's dose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChroniclePhysicalRowsDto {
    pub(crate) visibility_rows: u64,
    pub(crate) message_rows: u64,
}
