//! Untrusted mirror of [`RecordKind`](crate::observation::record_kind::RecordKind).

use serde::Deserialize;

/// The wire form of a record's kind: the externally-tagged `"Manifest"` unit variant or
/// `{"Dose": <n>}`, where the dose is the transparent 1-based ladder index as a bare number. This is
/// the discriminant [`WireRecordDto`](super::wire_record_dto::WireRecordDto) dispatches its body on.
/// The dose number is not range-checked here; `validate` proves it is a genuine `1..=NUM_DOSES` index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum RecordKindDto {
    Manifest,
    Dose(u64),
}
