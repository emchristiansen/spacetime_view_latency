//! The identity of a single durable Pilot ledger record.

use serde::Serialize;

use crate::entity_owner_pilot::pilot_record_kind::PilotRecordKind;
use crate::observation::record_seq::RecordSeq;

/// Names one record in the Pilot's durable stream by its ledger-global sequence and kind.
///
/// The sequence type is [`RecordSeq`] reused verbatim from the historical observation module: it is
/// ontology-free (a monotonic `u64` with a checked successor) and already carries exactly the
/// meaning needed here, so minting a parallel sequence type would duplicate a contract for no gain.
/// Only the *kind* is candidate-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct PilotRecordId {
    seq: RecordSeq,
    kind: PilotRecordKind,
}

impl PilotRecordId {
    pub(crate) fn new(seq: RecordSeq, kind: PilotRecordKind) -> Self {
        Self { seq, kind }
    }

    pub(crate) fn seq(self) -> RecordSeq {
        self.seq
    }

    pub(crate) fn kind(self) -> PilotRecordKind {
        self.kind
    }
}
