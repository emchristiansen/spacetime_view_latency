//! The durable append-only NDJSON ledger for the Pilot stage.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::entity_owner_pilot::pilot_record::PilotRecord;
use crate::entity_owner_pilot::pilot_record_id::PilotRecordId;
use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::file_line_writer::FileLineWriter;
use crate::observation::output_path::OutputPath;
use crate::observation::record_seq::RecordSeq;

/// The single durable NDJSON stream every Pilot record is appended to, one JSON object per line.
///
/// This is a candidate-specific sink, not a reuse of
/// [`ObservationSink`](crate::observation::observation_sink::ObservationSink): that sink's typed
/// write methods take a `DoseObservation`/`ValidatedRunManifest`, which are built from the historical
/// campaign's `RunDataset`/`CampaignDataset`/`RecordKind` ontology the spec preserves rather than
/// reinterprets. What *is* reused is everything below the record schema — the
/// [`DurableLineWriter`] seam, the [`FileLineWriter`] POSIX durability implementation, the
/// [`OutputPath`] newtype, and [`RecordSeq`] — so the two sinks share one durability implementation
/// and only their record vocabularies differ.
///
/// **Terminal on any persist failure**, exactly as the historical sink is. The moment a write fails,
/// the sink retains the reason and refuses every later write, re-reporting that original reason. No
/// successor reuses the failed sequence, so a torn or ambiguous tail can never be silently extended
/// with a line that looks well-formed. A driver that sees a refusal records the remaining attempts
/// [`NotRun`](super::attempt_outcome::AttemptOutcome::NotRun) rather than measuring into a ledger
/// that cannot receive the result.
///
/// A successful `sync_data` return is operational evidence a record is durable; it is not physical
/// crash proof, which is the platform honoring the fsync contract.
pub(crate) struct PilotSink {
    writer: Box<dyn DurableLineWriter>,
    next_seq: RecordSeq,
    last_durable_seq: Option<RecordSeq>,
    /// Set the first time a persist fails; once set, the sink is terminal and refuses further
    /// writes.
    poisoned: Option<String>,
}

/// One line of the ledger: the assigned identity followed by the record body. Every line flows
/// through this one wrapper, so the identity is serialized into each line identically rather than
/// only held in the sink's in-memory marker.
#[derive(Serialize)]
struct PilotLine<'a> {
    record: PilotRecordId,
    body: &'a PilotRecord<'a>,
}

impl PilotSink {
    /// Create the ledger file exclusively (`O_EXCL` via [`FileLineWriter::create`]) and wrap it in
    /// the durable writer. An existing path is a fail-fast error rather than a truncation, so a
    /// second Pilot run can never overwrite a prior run's evidence.
    pub(crate) fn create(output: &OutputPath) -> Result<Self> {
        let writer = FileLineWriter::create(output)
            .map_err(|e| anyhow!("creating the Pilot ledger: {}", e.diagnostic()))?;
        Ok(Self {
            writer: Box::new(writer),
            next_seq: RecordSeq::zero(),
            last_durable_seq: None,
            poisoned: None,
        })
    }

    /// Durably append one record, returning the identity it was written at. The sequence and
    /// last-durable marker advance only after the writer's write/flush/`sync_data` all return
    /// success; any failure poisons the sink before returning.
    pub(crate) fn write(&mut self, record: &PilotRecord) -> Result<PilotRecordId> {
        if let Some(original) = self.poisoned.as_ref() {
            return Err(anyhow!(
                "the Pilot ledger is terminal after an earlier persist failure and refuses further \
                 writes: {original}"
            ));
        }

        let id = PilotRecordId::new(self.next_seq, record.kind());
        let line = PilotLine { record: id, body: record };

        let mut bytes = match serde_json::to_vec(&line) {
            Ok(bytes) => bytes,
            Err(e) => {
                let diagnostic = format!(
                    "serializing Pilot ledger record seq {} ({:?}): {e}",
                    id.seq().get(),
                    id.kind()
                );
                self.poisoned = Some(diagnostic.clone());
                return Err(anyhow!(diagnostic));
            }
        };
        bytes.push(b'\n');

        if let Err(e) = self.writer.write_line(&bytes) {
            let diagnostic = format!(
                "persisting Pilot ledger record seq {} ({:?}); its durability is ambiguous: {e}",
                id.seq().get(),
                id.kind()
            );
            self.poisoned = Some(diagnostic.clone());
            return Err(anyhow!(diagnostic));
        }

        self.next_seq = id.seq().next();
        self.last_durable_seq = Some(id.seq());
        Ok(id)
    }

    /// Whether a prior persist failure has made the sink terminal, and why.
    pub(crate) fn poisoned(&self) -> Option<&str> {
        self.poisoned.as_deref()
    }

    /// The sequence of the last record whose durable write returned success, if any.
    pub(crate) fn last_durable_seq(&self) -> Option<RecordSeq> {
        self.last_durable_seq
    }

    /// Finalize the ledger, syncing the file metadata and its directory entry. Finalization is
    /// best-effort and runs even when the sink is poisoned, to preserve records already written; a
    /// retained poison reason and a fresh sync failure are both surfaced.
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
