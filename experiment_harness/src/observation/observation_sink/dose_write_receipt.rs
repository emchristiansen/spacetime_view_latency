//! The sealed receipt returned when a dose observation line completes the writer's durability contract.

use crate::dataset::dose_index::DoseIndex;
use crate::observation::record_id::RecordId;

/// What a successful observation write yields: the exact assigned [`RecordId`] the dose line was
/// written at (its campaign-global sequence paired with the `Dose` kind) and the [`DoseIndex`] the
/// written observation carried. Both are bound together here so a run cursor advancing its dose ladder
/// checks the *actual written* record and dose, never a caller-claimed sequence or dose number.
///
/// The constructor is `pub(super)`: only the enclosing sink module mints a receipt, and it does so
/// only after `ObservationSink::write_observation`'s persist returns success. No arbitrary crate
/// sibling can fabricate one — a run cursor cannot be advanced by a forged dose write.
///
/// Holding a receipt is operational evidence only: the observation line's full bytes completed the
/// writer's write + flush + `sync_data` contract (its call returned success). It is not a claim of
/// physical crash durability, nor of the samples' measurement provenance.
///
/// **Affine, not `Clone`:** a receipt is consumed exactly once to advance a run cursor's dose ladder.
/// It is deliberately not clonable — duplicating it would let one durable write advance two doses,
/// reintroducing the replay the ladder's exact-once accounting exists to prevent.
#[derive(Debug)]
pub(crate) struct DoseWriteReceipt {
    record: RecordId,
    dose: DoseIndex,
}

impl DoseWriteReceipt {
    /// Mint a receipt for a dose line whose write returned success. `pub(super)` so only the sink
    /// module can construct one.
    pub(super) fn new(record: RecordId, dose: DoseIndex) -> Self {
        Self { record, dose }
    }

    /// The full record identity the dose line was written at — its assigned sequence and `Dose` kind.
    pub(crate) fn record(&self) -> RecordId {
        self.record
    }

    /// The ladder index of the observation this receipt attests was durably written.
    pub(crate) fn dose(&self) -> DoseIndex {
        self.dose
    }
}
