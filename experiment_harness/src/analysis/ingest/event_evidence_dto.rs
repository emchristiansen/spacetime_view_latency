//! Untrusted mirror of [`EventEvidence`](crate::observation::event_evidence::EventEvidence).

use serde::Deserialize;

/// The wire form of a dose's SDK logical delivery-event evidence: the signed net delivered row delta
/// and the separate insert/delete/update counts. The trusted type enforces the identity
/// `inserts − deletes = net delta` in its constructor; the wire carries the four numbers
/// independently, so `validate` re-proves that identity rather than assuming it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EventEvidenceDto {
    pub(crate) delivered_net_row_delta: i64,
    pub(crate) inserts: u64,
    pub(crate) deletes: u64,
    pub(crate) updates: u64,
}
