//! The required durable NDJSON output sink for observation records.

use serde::Serialize;

mod manifest_write_receipt;

pub(crate) use manifest_write_receipt::ManifestWriteReceipt;

use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::dose_observation::DoseObservation;
use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::file_line_writer::FileLineWriter;
use crate::observation::finalize_error::FinalizeError;
use crate::observation::manifest_reference::ManifestReference;
use crate::observation::output_path::OutputPath;
use crate::observation::persist_error::PersistError;
use crate::observation::poisoned_tail::PoisonedTail;
use crate::observation::record_id::RecordId;
use crate::observation::record_kind::RecordKind;
use crate::observation::record_seq::RecordSeq;
use crate::observation::sink_create_error::SinkCreateError;
use crate::observation::tail_state::TailState;

/// The single durable NDJSON stream every run's records are appended to, one JSON object per line.
///
/// The sink owns the two record kinds' serialization and the sequencing/poisoning logic, driving a
/// [`DurableLineWriter`] for the actual durability I/O (the production writer is a
/// [`FileLineWriter`]; tests inject a scripted failing writer). Records are written through typed
/// methods that serialize *borrowed* wrappers ([`Self::write_manifest`], [`Self::write_observation`])
/// rather than an owning record enum, so a caller that only borrows a manifest never has to hand
/// ownership to the sink. The manifest record's [`ManifestReference`] is derived inside the sink from
/// the manifest itself and returned, so a run session keys its observations to exactly the reference
/// that was written.
///
/// Each write serializes one line, appends a newline, and hands it to the writer, which writes,
/// flushes, and `sync_data`s it; the global [`RecordSeq`] and the last-durable marker advance only
/// *after* that returns success. A failure is a typed [`PersistError`] distinguishing a pre-write
/// (serialization) failure — the record is definitely absent — from a write/flush/sync failure — the
/// record's durability is ambiguous.
///
/// **Terminal on any persist failure.** The moment any write fails, the sink records a
/// [`PoisonedTail`] and becomes terminal: every later write is refused (re-reporting the original
/// tail state, so no successor reuses the failed record's [`RecordSeq`] and appends a torn/duplicate
/// line), and [`Self::finalize`] surfaces the poison reason. This makes a torn or ambiguous tail
/// unable to be silently extended. A successful `sync_data` return is operational evidence the record
/// is durable; it is not physical crash proof, which is the platform honoring the fsync contract.
///
/// Durability boundary (POSIX) is exercised by the [`FileLineWriter`]: parent-directory fsync at
/// creation, `sync_data` per line, and file + directory sync at finalize.
pub(crate) struct ObservationSink {
    writer: Box<dyn DurableLineWriter>,
    next_seq: RecordSeq,
    last_durable_seq: Option<RecordSeq>,
    /// Set the first time a persist fails; once set, the sink is terminal and refuses further writes.
    poisoned: Option<PoisonedTail>,
}

/// One line of the required NDJSON output: the assigned [`RecordId`] (sequence and kind) followed by
/// the record body. Every line kind — and every test body — flows through this one wrapper, so the
/// assigned record identity is serialized into each line the same way for all of them, not only held
/// in the sink's in-memory marker. Serialization records the identity in the output; it does not by
/// itself make that identity durable.
#[derive(Serialize)]
struct RecordLine<B: Serialize> {
    record: RecordId,
    body: B,
}

/// The body of a manifest record line: the derived reference plus the immutable manifest.
#[derive(Serialize)]
struct ManifestBody<'a> {
    reference: &'a ManifestReference,
    manifest: &'a ValidatedRunManifest,
}

/// The body of a dose observation record line.
#[derive(Serialize)]
struct ObservationBody<'a> {
    observation: &'a DoseObservation,
}

impl ObservationSink {
    /// Create the required output file exclusively and wrap it in a durable file writer. The typed
    /// [`SinkCreateError`] distinguishes a failure that created no file from one that left the file
    /// in place with ambiguous presence durability.
    pub(crate) fn create(output: &OutputPath) -> std::result::Result<Self, SinkCreateError> {
        Ok(Self::with_writer(Box::new(FileLineWriter::create(output)?)))
    }

    /// Wrap an already-constructed durable writer. Private so production can only reach a sink through
    /// [`Self::create`] and the one required output file — an arbitrary writer must never be able to
    /// mint durable records outside that path.
    fn with_writer(writer: Box<dyn DurableLineWriter>) -> Self {
        Self {
            writer,
            next_seq: RecordSeq::zero(),
            last_durable_seq: None,
            poisoned: None,
        }
    }

    /// The sequence the next record will be assigned — i.e. the record currently being attempted.
    /// Used to name an ambiguous in-flight write in a frontier.
    pub(crate) fn pending_seq(&self) -> RecordSeq {
        self.next_seq
    }

    /// The sequence of the last record whose durable write returned success, if any.
    pub(crate) fn last_durable_seq(&self) -> Option<RecordSeq> {
        self.last_durable_seq
    }

    /// The retained poison reason, if a prior persist failure made the sink terminal.
    pub(crate) fn poisoned(&self) -> Option<&PoisonedTail> {
        self.poisoned.as_ref()
    }

    /// Durably append the immutable run manifest record. Returns a [`ManifestWriteReceipt`] carrying
    /// both the derived [`ManifestReference`] this run's observations key to and the assigned record
    /// identity the manifest line was written at, so the run owner keeps that identity rather than
    /// reconstructing it from mutable sink state. The receipt is minted only after the persist returns
    /// success.
    pub(crate) fn write_manifest(
        &mut self,
        manifest: &ValidatedRunManifest,
    ) -> std::result::Result<ManifestWriteReceipt, PersistError> {
        let reference = ManifestReference::of(manifest);
        let record = RecordId::new(self.next_seq, RecordKind::Manifest);
        let seq = self.persist(
            record,
            &RecordLine {
                record,
                body: ManifestBody {
                    reference: &reference,
                    manifest,
                },
            },
        )?;
        Ok(ManifestWriteReceipt::new(reference, seq))
    }

    /// Durably append one dose observation record, tagged with its ladder index.
    pub(crate) fn write_observation(
        &mut self,
        observation: &DoseObservation,
    ) -> std::result::Result<RecordSeq, PersistError> {
        let record = RecordId::new(self.next_seq, RecordKind::Dose(observation.dose()));
        self.persist(
            record,
            &RecordLine {
                record,
                body: ObservationBody { observation },
            },
        )
    }

    /// Serialize one line and persist it durably through the writer. If the sink is already poisoned,
    /// the write is refused up front, re-reporting the original tail state so no successor reuses the
    /// failed [`RecordSeq`]. Otherwise the sequence and last-durable marker advance only after the
    /// writer returns success; any failure poisons the sink before returning.
    fn persist<L: Serialize>(
        &mut self,
        record: RecordId,
        line: &L,
    ) -> std::result::Result<RecordSeq, PersistError> {
        // Refuse up front if already poisoned: the successor wrote nothing and carries the original
        // reason, so higher layers never mistake a refusal for a fresh attempt and no successor
        // reuses the failed `RecordSeq`.
        if let Some(tail) = self.poisoned.as_ref() {
            return Err(PersistError::SinkPoisoned {
                original: tail.clone(),
            });
        }

        let mut bytes = match serde_json::to_vec(line) {
            Ok(bytes) => bytes,
            Err(e) => {
                let diagnostic = format!("serializing a sink record: {e}");
                self.poisoned = Some(PoisonedTail::new(
                    record,
                    TailState::DefinitelyNotWritten,
                    diagnostic.clone(),
                ));
                return Err(PersistError::BeforeWrite { record, diagnostic });
            }
        };
        bytes.push(b'\n');
        if let Err(e) = self.writer.write_line(&bytes) {
            let diagnostic = format!("persisting a sink record line: {e}");
            self.poisoned = Some(PoisonedTail::new(
                record,
                TailState::DurabilityAmbiguous,
                diagnostic.clone(),
            ));
            return Err(PersistError::DurabilityAmbiguous { record, diagnostic });
        }
        self.next_seq = record.seq().next();
        self.last_durable_seq = Some(record.seq());
        Ok(record.seq())
    }

    /// Finalize the sink. Finalization is always best-effort: the writer's file + directory sync runs
    /// even when the sink is poisoned, to preserve records and metadata already written. The typed
    /// [`FinalizeError`] aggregates both the prior poison reason (if any) and a fresh sync failure (if
    /// the sync failed), returning clean success only when neither is present.
    pub(crate) fn finalize(mut self) -> std::result::Result<(), FinalizeError> {
        let sync_failures = self.writer.finalize().err();
        FinalizeError::combine(self.poisoned, sync_failures)
    }
}

#[cfg(test)]
impl ObservationSink {
    /// Inject a scripted durable writer to drive the sink's state transitions. Test-only: production
    /// reaches a sink solely through [`Self::create`] and the one required output file.
    pub(crate) fn from_writer(writer: Box<dyn DurableLineWriter>) -> Self {
        Self::with_writer(writer)
    }

    /// Persist an arbitrary serializable payload under the next [`RecordSeq`], exercising the same
    /// `persist` path (sequence advance, poisoning, refusal) as the real record writers. Lets tests
    /// drive the sink's state transitions — including the pre-write path via a deliberately failing
    /// `Serialize` value — without constructing a full `DoseObservation`, whose summary derivation is
    /// not yet implemented.
    pub(crate) fn persist_test_record<L: Serialize>(
        &mut self,
        line: &L,
    ) -> std::result::Result<RecordSeq, PersistError> {
        let record = RecordId::new(self.next_seq, RecordKind::Manifest);
        self.persist(record, line)
    }
}

#[cfg(test)]
mod tests;
