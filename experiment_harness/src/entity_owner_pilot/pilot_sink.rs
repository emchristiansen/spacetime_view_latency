//! The durable append-only NDJSON ledger for the Pilot stage.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::entity_owner_pilot::pilot_record::PilotRecord;
use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::file_line_writer::FileLineWriter;
use crate::observation::output_path::OutputPath;
use crate::observation::record_seq::RecordSeq;

/// The single durable NDJSON stream every Pilot record is appended to, one JSON object per line.
///
/// Candidate-specific rather than a reuse of
/// [`ObservationSink`](crate::observation::observation_sink::ObservationSink), whose write methods
/// take a `DoseObservation`/`ValidatedRunManifest` built from the historical campaign ontology. What
/// *is* reused is everything below the record schema — the [`DurableLineWriter`] seam, the
/// [`FileLineWriter`] POSIX durability implementation, [`OutputPath`], and [`RecordSeq`] — so both
/// sinks share one durability implementation and differ only in vocabulary.
///
/// **Terminal on any persist failure**, as the historical sink is: the first failure is retained and
/// every later write refused, re-reporting that reason. No successor reuses the failed sequence, so
/// a torn or ambiguous tail cannot be silently extended with a well-formed-looking line.
///
/// A successful `sync_data` is operational evidence of durability, not physical crash proof.
pub(crate) struct PilotSink {
    writer: Box<dyn DurableLineWriter>,
    next_seq: RecordSeq,
    /// Set by the first persist failure; thereafter the sink is terminal.
    poisoned: Option<String>,
}

/// One ledger line: the assigned sequence, then the record body.
///
/// The body's kind is not repeated here — serde's external tagging emits the [`PilotRecord`] variant
/// name in the line, so a separate kind field would be a second copy of the same fact that could
/// disagree with it.
#[derive(Serialize)]
struct PilotLine<'a> {
    seq: RecordSeq,
    body: &'a PilotRecord<'a>,
}

impl PilotSink {
    /// Create the ledger file exclusively (`O_EXCL` via [`FileLineWriter::create`]), so a rerun
    /// fails fast rather than truncating a prior run's evidence.
    pub(crate) fn create(output: &OutputPath) -> Result<Self> {
        let writer = FileLineWriter::create(output)
            .map_err(|e| anyhow!("creating the Pilot ledger: {}", e.diagnostic()))?;
        Ok(Self::with_writer(Box::new(writer)))
    }

    /// Wrap an already-constructed durable writer. Private so production can only reach a sink
    /// through [`Self::create`] and the one required output file — an arbitrary writer must never be
    /// able to stand in for the ledger. Mirrors
    /// [`ObservationSink::with_writer`](crate::observation::observation_sink::ObservationSink).
    fn with_writer(writer: Box<dyn DurableLineWriter>) -> Self {
        Self {
            writer,
            next_seq: RecordSeq::zero(),
            poisoned: None,
        }
    }

    /// Durably append one record, returning the sequence it was written at. The sequence advances
    /// only after write, flush, and `sync_data` all return success; any failure poisons the sink.
    pub(crate) fn write(&mut self, record: &PilotRecord) -> Result<RecordSeq> {
        if let Some(original) = self.poisoned.as_ref() {
            return Err(anyhow!(
                "the Pilot ledger is terminal after an earlier persist failure and refuses further \
                 writes: {original}"
            ));
        }

        let seq = self.next_seq;
        let line = PilotLine { seq, body: record };

        let mut bytes = match serde_json::to_vec(&line) {
            Ok(bytes) => bytes,
            Err(e) => {
                let diagnostic = format!(
                    "serializing Pilot ledger record seq {} ({}): {e}",
                    seq.get(),
                    record.variant_name()
                );
                self.poisoned = Some(diagnostic.clone());
                return Err(anyhow!(diagnostic));
            }
        };
        bytes.push(b'\n');

        if let Err(e) = self.writer.write_line(&bytes) {
            let diagnostic = format!(
                "persisting Pilot ledger record seq {} ({}); its durability is ambiguous: {e}",
                seq.get(),
                record.variant_name()
            );
            self.poisoned = Some(diagnostic.clone());
            return Err(anyhow!(diagnostic));
        }

        self.next_seq = seq.next();
        Ok(seq)
    }

    /// Finalize the ledger, syncing file metadata and the directory entry. Best-effort: the sync
    /// runs even when poisoned, to preserve records already written, and both the retained poison
    /// reason and a fresh sync failure are surfaced.
    pub(crate) fn finalize(mut self) -> Result<()> {
        let sync = self
            .writer
            .finalize()
            .map_err(|failures| anyhow!("syncing the Pilot ledger at finalize: {failures:?}"));

        match (self.poisoned, sync) {
            (None, Ok(())) => Ok(()),
            (None, Err(e)) => Err(e),
            (Some(original), Ok(())) => Err(anyhow!(
                "the Pilot ledger was terminal before finalize: {original}"
            )),
            (Some(original), Err(e)) => Err(e).context(format!(
                "the Pilot ledger was also terminal before finalize: {original}"
            )),
        }
    }
}

#[cfg(test)]
impl PilotSink {
    /// Inject a durable writer to observe the exact lines the ledger emits. Test-only: production
    /// reaches a sink solely through [`Self::create`] and the one required output file. Mirrors
    /// [`ObservationSink::from_writer`](crate::observation::observation_sink::ObservationSink).
    pub(crate) fn from_writer(writer: Box<dyn DurableLineWriter>) -> Self {
        Self::with_writer(writer)
    }
}

#[cfg(test)]
mod tests;
