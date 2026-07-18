//! Untrusted mirror of [`RecordId`](crate::observation::record_id::RecordId).

use serde::Deserialize;

use crate::analysis::ingest::record_kind_dto::RecordKindDto;

/// The wire form of a record's identity: its transparent campaign-global sequence (a bare number) and
/// its kind. The sequence is not checked for contiguity here; `validate` proves the sequence set is
/// exactly `0..N` without gaps or duplicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordIdDto {
    pub(crate) seq: u64,
    pub(crate) kind: RecordKindDto,
}
