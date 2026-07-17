//! A run's owned output sink and bound context, paired with the receipt proving its manifest was written.

use crate::dataset::run_dataset::RunDataset;
use crate::observation::observation_sink::ManifestWriteReceipt;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::persist_error::PersistError;

/// A run whose immutable manifest has been durably written: the campaign's [`ObservationSink`], the run's
/// bound [`RunDataset`] context, and the [`ManifestWriteReceipt`] the sink returned for *that exact write* —
/// all three owned as one value.
///
/// The only way to obtain one is [`Self::write`], which takes the owned sink and context, performs the real
/// `sink.write_manifest(context.manifest())` internally, and — on success — returns all three bound
/// together. There is no constructor that accepts a `context` and a separately supplied `receipt`, and,
/// because the sink is *absorbed into this value*, there is no later transition that pairs a loose sink with
/// a `WrittenRun`. So neither a receipt/context mismatch (receipt minted for manifest A carried with context
/// B) nor an output-stream mismatch (a sink other than the one that performed the write) is representable
/// anywhere downstream: a receipt and a sink only ever exist alongside the exact context whose manifest the
/// sink wrote to produce that receipt.
///
/// **Sealed to the manifest-writer subtree.** `WrittenRun`, [`Self::write`], and [`Self::into_parts`] are
/// visible only within the [`run_writing_manifest`](super) subtree (`write` is `pub(super)`; the type and
/// its accessors are `pub(in …run_writing_manifest)`). Nothing *outside* that subtree — no `run_cursor`
/// sibling, not the run driver in `campaign`, not the wider crate — can name `WrittenRun`, mint one, or
/// recover its sink and context: the sink and the manifest receipt are inaccessible past the sealed
/// boundary. Rust `pub(super)` necessarily extends to a module's descendants, so the pre-dose successor
/// states (which lie in this subtree and hold a `WrittenRun`) are *inside* that visibility region rather
/// than walled off from it — this is a subtree-level seal, not a per-state exclusivity claim. Within the
/// subtree the states are implemented to expose only the linear forward transitions below, with no remint or
/// reconstruction path; the seam the design closes is the externally-exposed one, so a `run_cursor`- or
/// crate-level caller can neither forge a `WrittenRun` nor skip the manifest write.
///
/// The receipt's role is durable-progress evidence only: its
/// [`RecordId`](crate::observation::record_id::RecordId) seeds the dosing last-record chain
/// ([`RunDosing::begin`](super::run_seeding_background::RunDosing)) and is the last-successful record of
/// every pre-dose stop frontier. Observations do **not** key to the receipt — they derive their manifest
/// reference from `context.manifest()` — so once dosing begins the receipt has served its purpose and only
/// the sink and context are carried on ([`Self::into_parts`]).
pub(in crate::campaign::run_cursor::run_writing_manifest) struct WrittenRun {
    sink: ObservationSink,
    context: RunDataset,
    receipt: ManifestWriteReceipt,
}

impl WrittenRun {
    /// Write `context`'s manifest through the owned `sink` and, on success, return the sink, context, and
    /// *that write's own* receipt bound together. The receipt is never supplied by the caller — it is minted
    /// here from the exact manifest just written — and the sink is taken by value and absorbed, so neither
    /// pairing can be forged. On a sink-write failure the context is dropped and the recovered sink is
    /// returned alongside the [`PersistError`], so the caller
    /// ([`RunWritingManifest`](super::RunWritingManifest)) — which derived the failure coordinate from the
    /// context *before* invoking this — can build the typed stopped terminal from the same sink.
    /// `pub(super)`, confining the mint to the `run_writing_manifest` subtree — no caller outside it can
    /// name `WrittenRun` or perform the real write. (`pub(super)` also reaches the parent's descendants, so
    /// this is a subtree-level seal; the implemented mint call site is the manifest-writing transition.)
    pub(super) fn write(
        mut sink: ObservationSink,
        context: RunDataset,
    ) -> std::result::Result<Self, (ObservationSink, PersistError)> {
        match sink.write_manifest(context.manifest()) {
            Ok(receipt) => Ok(Self {
                sink,
                context,
                receipt,
            }),
            Err(error) => Err((sink, error)),
        }
    }

    /// The bound run context (manifest + resolved dataset) — the single carrier of this run's identity.
    pub(in crate::campaign::run_cursor::run_writing_manifest) fn context(&self) -> &RunDataset {
        &self.context
    }

    /// The receipt for this run's manifest write — the durable-progress evidence the pre-dose stop frontiers
    /// and the initial dosing last-record are drawn from.
    pub(in crate::campaign::run_cursor::run_writing_manifest) fn receipt(
        &self,
    ) -> &ManifestWriteReceipt {
        &self.receipt
    }

    /// Consume the bundle into its owned sink and context, dropping the receipt once it has served its
    /// purpose (seeding the dosing last-record chain, or being read for a pre-dose stop frontier). The single
    /// owned sink threads on — never paired anew with a separately held context — and past this point the
    /// durable-progress marker is the evolving dosing last-record.
    pub(in crate::campaign::run_cursor::run_writing_manifest) fn into_parts(
        self,
    ) -> (ObservationSink, RunDataset) {
        (self.sink, self.context)
    }
}
