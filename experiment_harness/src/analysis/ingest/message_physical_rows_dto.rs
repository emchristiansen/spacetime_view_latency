//! Untrusted mirror of [`MessagePhysicalRows`](crate::dataset::message_physical_rows::MessagePhysicalRows).

use serde::Deserialize;

/// The wire form of the single-table `message` family's physical row count. A bare count; `validate`
/// proves it equals the preregistered physical cardinality for the run's dose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MessagePhysicalRowsDto {
    pub(crate) message_rows: u64,
}
