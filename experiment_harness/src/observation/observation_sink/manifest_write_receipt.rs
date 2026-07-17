//! The sealed receipt returned when the run manifest line completes the writer's durability contract.

use crate::observation::manifest_reference::ManifestReference;
use crate::observation::record_id::RecordId;
use crate::observation::record_kind::RecordKind;
use crate::observation::record_seq::RecordSeq;

/// What a successful manifest write yields: the derived [`ManifestReference`] this run's observations
/// key to, and the assigned [`RecordSeq`] the manifest line was written at. The record kind is fixed
/// to `Manifest` by this type — only a sequence is stored, and [`Self::record`] always pairs it with
/// [`RecordKind::Manifest`] — so a non-`Manifest` identity is unrepresentable.
///
/// The constructor is `pub(super)`: only the enclosing sink module mints a receipt, and it does so
/// only after `ObservationSink::write_manifest`'s persist returns success. No arbitrary crate sibling
/// can fabricate one. Cloning an already-valid receipt is fine; fabricating a fresh one is not.
///
/// Holding a receipt is operational evidence only: the manifest line's full bytes completed the
/// writer's write + flush + `sync_data` contract (its call returned success). It is not a claim of
/// physical crash durability, nor of the manifest's provenance.
#[derive(Debug, Clone)]
pub(crate) struct ManifestWriteReceipt {
    reference: ManifestReference,
    seq: RecordSeq,
}

impl ManifestWriteReceipt {
    /// Mint a receipt for a manifest line whose write returned success. `pub(super)` so only the sink
    /// module can construct one.
    pub(super) fn new(reference: ManifestReference, seq: RecordSeq) -> Self {
        Self { reference, seq }
    }

    /// The immutable reference this run's observations key to.
    pub(crate) fn reference(&self) -> &ManifestReference {
        &self.reference
    }

    /// The assigned sequence the manifest line was written at.
    pub(crate) fn seq(&self) -> RecordSeq {
        self.seq
    }

    /// The full record identity — the assigned sequence paired with the fixed `Manifest` kind.
    pub(crate) fn record(&self) -> RecordId {
        RecordId::new(self.seq, RecordKind::Manifest)
    }
}
