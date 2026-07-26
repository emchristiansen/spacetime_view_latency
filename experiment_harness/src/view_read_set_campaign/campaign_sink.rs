//! The durable append-only NDJSON ledger for the fresh-server campaign.

use anyhow::Result;

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::output_path::OutputPath;
use crate::observation::record_seq::RecordSeq;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;

/// The single durable NDJSON stream every campaign record is appended to, one JSON object per line.
///
/// Copies [`PilotSink`](crate::entity_owner_pilot::pilot_sink::PilotSink), which this repository has
/// already run a real campaign through: the same [`DurableLineWriter`] seam, the same
/// [`FileLineWriter`](crate::observation::file_line_writer::FileLineWriter) POSIX durability
/// implementation, the same [`OutputPath`] and [`RecordSeq`], and the same three-field shape. It is
/// a separate sink for the same reason the record vocabulary is separate: the write methods differ
/// only in which record type they accept, and sharing one would mean one ledger schema for two
/// campaigns.
///
/// **The sink owns sequence assignment.** [`Self::append`] takes a [`CampaignRecord`] and returns
/// the [`RecordSeq`] it was written at; there is no parameter for a caller to pass one. A ledger's
/// sequences are meaningful only if they are contiguous in write order, and
/// [`ReconciledCampaign::reconciled`](super::reconciled_campaign::ReconciledCampaign::reconciled)
/// rejects a stream whose sequences have a hole — so a caller able to choose a number could produce
/// a ledger that fails its own reconciliation, or worse, one that passes while describing a write
/// order that never happened. That
/// [`CampaignLedgerLine::at`](super::campaign_ledger_line::CampaignLedgerLine::at) is callable
/// elsewhere does not weaken this: a hand-built line is not a written line, and this sink is the
/// only thing that writes.
///
/// **Terminal on any persist failure**, as the Pilot's sink is: the first failure is retained and
/// every later write refused, re-reporting that reason. No successor reuses the failed sequence, so
/// a torn or ambiguous tail cannot be silently extended with a well-formed-looking line. That
/// discipline is what makes the spec's poisoned-ledger rule implementable — a ledger that cannot
/// persist also cannot append a truthful record about its own failure, so the campaign stops and the
/// frozen inventory already on disk is what makes every missing terminal slot detectable.
///
/// A successful `sync_data` is operational evidence of durability, not physical crash proof.
///
/// **Phase 1 boundary.** Every behavior is an explicit `todo!()`: opening the file, serializing,
/// writing, poisoning, and finalizing. What is fixed here is the shape — which types the ledger
/// accepts, who assigns sequences, and that a failure is terminal rather than skippable.
pub(crate) struct CampaignSink {
    writer: Box<dyn DurableLineWriter>,
    next_seq: RecordSeq,
    /// Set by the first persist failure; thereafter the sink is terminal.
    poisoned: Option<String>,
}

impl CampaignSink {
    /// Create the ledger file exclusively, so a rerun fails fast rather than truncating a prior
    /// run's evidence.
    ///
    /// **Phase 1 boundary.** Opening the file is persistence behavior and is stubbed; with it
    /// stubbed there is no path to a `CampaignSink` at all, which is the intended Phase-1 state.
    pub(crate) fn create(output: &OutputPath) -> Result<Self> {
        let _ = output;
        todo!(
            "Phase 2: open the ledger through FileLineWriter::create, which is O_EXCL, so an \
             existing path is a failure rather than a truncation; start the sequence at \
             RecordSeq::zero and the poison at None. The writer-injection seam the Pilot's sink \
             exposes for its fake-writer tests arrives here with the tests that need it, so that \
             production still reaches a sink solely through this constructor and the one required \
             output file"
        )
    }

    /// Durably append one record, returning the sequence the sink assigned it.
    ///
    /// Takes the record by value because [`CampaignRecord`] is owned — unlike the Pilot's borrowing
    /// write vocabulary — so there is nothing to clone on the way to the line.
    ///
    /// **Phase 1 boundary.** The whole write path is stubbed.
    pub(crate) fn append(&mut self, record: CampaignRecord) -> Result<RecordSeq> {
        let _ = record;
        todo!(
            "Phase 2: refuse immediately if already poisoned, re-reporting the original reason; \
             otherwise take self.next_seq, build the line with CampaignLedgerLine::at, serialize it \
             with serde_json and a trailing newline, and write it through the DurableLineWriter. \
             Advance next_seq only after write, flush, and sync_data have all returned success. Any \
             serialization or persist failure poisons the sink with a diagnostic naming the sequence \
             and CampaignRecord::variant_name — the variant name rather than the rendered body, \
             because on a serialization failure the body cannot be rendered"
        )
    }

    /// Finalize the ledger, syncing file metadata and the directory entry.
    ///
    /// **Phase 1 boundary.** Stubbed.
    pub(crate) fn finalize(self) -> Result<()> {
        let _ = self;
        todo!(
            "Phase 2: best-effort — run the writer's finalize even when poisoned, so records \
             already written are preserved, and surface both the retained poison reason and any \
             fresh sync failure rather than letting either mask the other"
        )
    }
}
