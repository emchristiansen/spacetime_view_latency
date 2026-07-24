//! The identity of a single durable sink record: its global sequence and kind.

use serde::Serialize;

use crate::observation::record_kind::RecordKind;
use crate::observation::record_seq::RecordSeq;

/// Names one record in the durable stream by its campaign-global sequence and kind. Used by a
/// frontier to identify the record whose write was in flight when a failure made the sink tail's
/// durability ambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct RecordId {
    seq: RecordSeq,
    kind: RecordKind,
}

impl RecordId {
    pub(crate) fn new(seq: RecordSeq, kind: RecordKind) -> Self {
        Self { seq, kind }
    }

    pub(crate) fn seq(self) -> RecordSeq {
        self.seq
    }

    pub(crate) fn kind(self) -> RecordKind {
        self.kind
    }
}
