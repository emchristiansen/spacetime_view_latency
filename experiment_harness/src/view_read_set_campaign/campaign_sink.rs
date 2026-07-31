//! The durable append-only NDJSON ledger for the fresh-server campaign.
//!
//! The sink lives in the private, *childless* inline module [`sealed`] because its whole discipline
//! is state no caller may set: the next sequence to assign, and the poison that makes a failure
//! terminal. A private field is visible to its declaring module **and every descendant**, so a
//! `#[cfg(test)] mod tests` child, or any child added later, could write the struct literal with an
//! arbitrary `next_seq` or a cleared `poisoned` — producing a ledger whose sequences describe a write
//! order that never happened, or one that silently extends a torn tail. The tests therefore live in
//! a module that is a *sibling* of [`sealed`], never a child: they drive the sink only through its
//! own methods and the one injected writer seam below, so a sequence or poison state they observe is
//! one the sink really reached.
//!
//! That seam — `CampaignSink::from_writer`, `#[cfg(test)]` and inside `sealed` — is the single
//! deliberate exception, and it grants exactly what the Pilot's sink grants: a fresh sink at sequence
//! zero and unpoisoned, over a writer that performs no real I/O. It cannot name a starting sequence
//! or a poison reason, so it opens no door the struct literal would have opened.

mod sealed {
    use anyhow::{anyhow, Context, Result};

    use crate::observation::durable_line_writer::DurableLineWriter;
    use crate::observation::file_line_writer::FileLineWriter;
    use crate::observation::output_path::OutputPath;
    use crate::observation::record_seq::RecordSeq;
    use crate::view_read_set_campaign::campaign_ledger_line::CampaignLedgerLine;
    use crate::view_read_set_campaign::campaign_record::CampaignRecord;

    /// The single durable NDJSON stream every campaign record is appended to, one JSON object per
    /// line.
    ///
    /// Copies [`PilotSink`](crate::entity_owner_pilot::pilot_sink::PilotSink), which this repository
    /// has already run a real campaign through: the same [`DurableLineWriter`] seam, the same
    /// [`FileLineWriter`] POSIX durability implementation, the same [`OutputPath`] and [`RecordSeq`],
    /// and the same three-field shape. It is a separate sink for the same reason the record
    /// vocabulary is separate: the write methods differ only in which record type they accept, and
    /// sharing one would mean one ledger schema for two campaigns.
    ///
    /// **The sink owns sequence assignment.** [`Self::append`] takes a [`CampaignRecord`] and
    /// returns the [`RecordSeq`] it was written at; there is no parameter for a caller to pass one.
    /// A ledger's sequences are meaningful only if they are contiguous in write order, and
    /// `ReconciledCampaign::reconciled` rejects a stream whose sequences have a hole — so a caller
    /// able to choose a number could produce a ledger that fails its own reconciliation, or worse,
    /// one that passes while describing a write order that never happened. That
    /// [`CampaignLedgerLine::at`] is callable elsewhere does not weaken this: a hand-built line is not
    /// a written line, and this sink is the only thing that writes. Every field being private to this
    /// childless module is what makes that hold — the counter cannot be set from outside, only
    /// advanced by a successful write.
    ///
    /// **Terminal on any persist failure**, as the Pilot's sink is: the first failure is retained
    /// and every later write refused, re-reporting that reason. No successor reuses the failed
    /// sequence, so a torn or ambiguous tail cannot be silently extended with a well-formed-looking
    /// line. That discipline is what makes the spec's poisoned-ledger rule implementable — a ledger
    /// that cannot persist also cannot append a truthful record about its own failure, so the
    /// campaign stops and the frozen inventory already on disk is what makes every missing terminal
    /// slot detectable.
    ///
    /// **One poison string for two failure regimes.** A serialization failure means the record is
    /// definitely absent; a write/flush/`sync_data` failure leaves its durability ambiguous, because
    /// a full line may already be on disk. The historical
    /// [`ObservationSink`](crate::observation::observation_sink::ObservationSink) separates those
    /// with a typed [`TailState`](crate::observation::tail_state::TailState); this sink keeps the
    /// Pilot's single `Option<String>`, so the distinction survives only in the diagnostic's wording.
    /// That is a real narrowing, recorded here rather than silently repaired: both regimes are
    /// terminal, and nothing in this campaign branches on which one occurred.
    ///
    /// A successful `sync_data` is operational evidence of durability, not physical crash proof.
    pub(crate) struct CampaignSink {
        writer: Box<dyn DurableLineWriter>,
        next_seq: RecordSeq,
        /// Set by the first persist failure; thereafter the sink is terminal.
        poisoned: Option<String>,
    }

    impl CampaignSink {
        /// Create the ledger file exclusively (`O_EXCL` via [`FileLineWriter::create`]), so a rerun
        /// fails fast rather than truncating a prior run's evidence.
        ///
        /// The typed [`SinkCreateError`](crate::observation::sink_create_error::SinkCreateError) is
        /// flattened into the anyhow surface this sink's frozen signatures use, exactly as the
        /// Pilot's sink flattens it. Its two regimes — no file created by this call, versus a created
        /// file whose directory entry's durability is unknown — survive in the diagnostic text but
        /// not in the type. Both are fatal to a campaign that has not started, which is the only
        /// caller.
        pub(crate) fn create(output: &OutputPath) -> Result<Self> {
            let writer = FileLineWriter::create(output)
                .map_err(|e| anyhow!("creating the campaign ledger: {}", e.diagnostic()))?;
            Ok(Self::with_writer(Box::new(writer)))
        }

        /// Wrap an already-constructed durable writer at sequence zero, unpoisoned. Private so
        /// production can only reach a sink through [`Self::create`] and the one required output file
        /// — an arbitrary writer must never be able to stand in for the ledger. Mirrors
        /// [`PilotSink::with_writer`](crate::entity_owner_pilot::pilot_sink::PilotSink).
        fn with_writer(writer: Box<dyn DurableLineWriter>) -> Self {
            Self {
                writer,
                next_seq: RecordSeq::zero(),
                poisoned: None,
            }
        }

        /// Durably append one record, returning the sequence the sink assigned it. The sequence
        /// advances only after write, flush, and `sync_data` have all returned success; any failure
        /// poisons the sink.
        ///
        /// Takes the record by value because [`CampaignRecord`] is owned — unlike the Pilot's
        /// borrowing write vocabulary — so there is nothing to clone on the way to the line. That
        /// ownership is why the variant name is read *before* the record moves into
        /// [`CampaignLedgerLine::at`]: a serialization failure is exactly the case where the body
        /// cannot be rendered, so the diagnostic must name the discriminant, and by then the record
        /// itself is gone.
        pub(crate) fn append(&mut self, record: CampaignRecord) -> Result<RecordSeq> {
            if let Some(original) = self.poisoned.as_ref() {
                return Err(anyhow!(
                    "the campaign ledger is terminal after an earlier persist failure and refuses \
                     further writes: {original}"
                ));
            }

            let seq = self.next_seq;
            let variant = record.variant_name();
            let line = CampaignLedgerLine::at(seq, record);

            let mut bytes = match serde_json::to_vec(&line) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let diagnostic = format!(
                        "serializing campaign ledger record seq {} ({variant}): {e}",
                        seq.get()
                    );
                    self.poisoned = Some(diagnostic.clone());
                    return Err(anyhow!(diagnostic));
                }
            };
            bytes.push(b'\n');

            if let Err(e) = self.writer.write_line(&bytes) {
                let diagnostic = format!(
                    "persisting campaign ledger record seq {} ({variant}); its durability is \
                     ambiguous: {e}",
                    seq.get()
                );
                self.poisoned = Some(diagnostic.clone());
                return Err(anyhow!(diagnostic));
            }

            self.next_seq = seq.next();
            Ok(seq)
        }

        /// Finalize the ledger, syncing file metadata and the directory entry. Best-effort: the sync
        /// runs even when poisoned, to preserve records already written, and both the retained poison
        /// reason and a fresh sync failure are surfaced rather than either masking the other.
        ///
        /// Consumes the sink, so nothing can append after the final sync — the one lifetime rule the
        /// type enforces. It does not enforce that finalization *happens*: there is no `Drop`
        /// backstop, which is why the driver's entrypoint may not `?` past this sink's creation.
        pub(crate) fn finalize(mut self) -> Result<()> {
            let sync = self.writer.finalize().map_err(|failures| {
                anyhow!("syncing the campaign ledger at finalize: {failures:?}")
            });

            match (self.poisoned, sync) {
                (None, Ok(())) => Ok(()),
                (None, Err(e)) => Err(e),
                (Some(original), Ok(())) => Err(anyhow!(
                    "the campaign ledger was terminal before finalize: {original}"
                )),
                (Some(original), Err(e)) => Err(e).context(format!(
                    "the campaign ledger was also terminal before finalize: {original}"
                )),
            }
        }
    }

    #[cfg(test)]
    impl CampaignSink {
        /// Inject a durable writer to observe the exact lines the ledger emits and to drive its
        /// persist failures without a real I/O fault. Test-only: production reaches a sink solely
        /// through [`Self::create`] and the one required output file. Mirrors
        /// [`PilotSink::from_writer`](crate::entity_owner_pilot::pilot_sink::PilotSink).
        pub(crate) fn from_writer(writer: Box<dyn DurableLineWriter>) -> Self {
            Self::with_writer(writer)
        }
    }
}

pub(crate) use sealed::CampaignSink;

// A *sibling* of `sealed`, never a child — so these tests cannot write the struct literal, cannot
// set `next_seq`, and cannot clear `poisoned`. Every state they assert is one the sink reached
// through `create`, `append`, or the injected-writer constructor.
#[cfg(test)]
mod tests;
