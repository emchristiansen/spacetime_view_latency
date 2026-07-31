//! One sequenced line of the durable campaign ledger.

use serde::Serialize;

use crate::observation::record_seq::RecordSeq;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;

/// One ledger line: the sequence it was written at, then the record body.
///
/// The same `{seq, body}` shape the Pilot's sink emits, promoted from a private struct inside that
/// sink to a named type here because it is now read as well as written:
/// `ReconciledCampaign::reconciled` consumes a
/// stream of these, and the sequence is part of what it checks. A reconciliation handed bare
/// [`CampaignRecord`]s could not tell a ledger whose lines are contiguous and correctly ordered from
/// one with a hole in it.
///
/// The body's kind is not repeated beside the sequence — serde's external tagging emits the
/// [`CampaignRecord`] variant name in the line, so a separate kind field would be a second copy of
/// the same fact that could disagree with it.
///
/// **Who can construct it, and what that is worth.** Any code in the crate, through [`Self::at`].
/// That is the honest surface: a line is a transport shape, and the invariants that matter — one
/// `Inventory` first, contiguous sequences, one terminal per identity — are properties of a *whole
/// stream*, so none of them is checkable here. They are checked exactly once, in
/// `ReconciledCampaign::reconciled`, and this type
/// makes no claim to have anticipated any of them.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct CampaignLedgerLine {
    seq: RecordSeq,
    body: CampaignRecord,
}

impl CampaignLedgerLine {
    /// Pair a record with the sequence it occupies.
    pub(crate) fn at(seq: RecordSeq, body: CampaignRecord) -> Self {
        Self { seq, body }
    }

    /// This line's campaign-global position.
    pub(crate) fn seq(&self) -> RecordSeq {
        self.seq
    }

    /// The record body.
    pub(crate) fn body(&self) -> &CampaignRecord {
        &self.body
    }
}
