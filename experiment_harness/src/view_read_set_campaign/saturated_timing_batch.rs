//! The complete, lossless timing record of one saturated batch.

use anyhow::{anyhow, Result};
use serde::{Serialize, Serializer};

use crate::view_read_set_campaign::campaign_params::{
    CHANNEL_SAMPLE_COUNT, CHANNEL_SAMPLE_COUNT_USIZE,
};
use crate::view_read_set_campaign::saturated_write_timing::SaturatedWriteTiming;

/// Every write of one saturated batch, in issue order, each carrying its issue and confirmation
/// offsets from the batch's common origin.
///
/// The count is proven *structurally*, by the field's type rather than by who may construct one:
/// the field is a fixed-size array, so a batch cannot hold any other number of writes however it was
/// built — "exactly [`CHANNEL_SAMPLE_COUNT`]" is the type rather than a check someone must remember.
/// That is why this type is deliberately *not* confined to a private childless module the way its
/// element type and the validated evidence types are: [`Self::sealed`] adds only the conversion the
/// field already demands, so a struct literal written by some future child module of this file could
/// bypass nothing. The elements themselves remain unforgeable, because
/// [`SaturatedWriteTiming`] is sealed and admits no inverted pair. Mirrors
/// [`RawLatencies`](crate::observation::raw_latencies::RawLatencies)' treatment of the same count,
/// and differs from it in exactly the way the spec requires: this retains both offsets rather than
/// only their difference, so the FIFO and stable-issue-spacing interpretation stays falsifiable
/// against the record.
///
/// Issue order is the storage order, and is not re-derived from the offsets — the whole point is
/// that the offsets may contradict it, and a record that sorted itself could not show that.
///
/// Sealing proves the count and each write's internal consistency, and nothing more. That the
/// timings are genuine observations rather than fabricated numbers is a separate property,
/// established by the driver's measurement path — the same division
/// [`RawLatencies`](crate::observation::raw_latencies::RawLatencies) makes explicit.
#[derive(Debug, Clone)]
pub(crate) struct SaturatedTimingBatch {
    writes: Box<[SaturatedWriteTiming; CHANNEL_SAMPLE_COUNT_USIZE]>,
}

impl SaturatedTimingBatch {
    /// Seal a full batch's timings in issue order, failing loud unless exactly
    /// [`CHANNEL_SAMPLE_COUNT`] were collected.
    pub(crate) fn sealed(writes: Vec<SaturatedWriteTiming>) -> Result<Self> {
        let collected = writes.len();
        let writes: Box<[SaturatedWriteTiming; CHANNEL_SAMPLE_COUNT_USIZE]> =
            writes.into_boxed_slice().try_into().map_err(|_| {
                anyhow!(
                    "a saturated batch's timing record must contain exactly \
                     {CHANNEL_SAMPLE_COUNT} writes, got {collected}"
                )
            })?;
        Ok(Self { writes })
    }

    /// The lossless per-write timings in issue order, always exactly [`CHANNEL_SAMPLE_COUNT`] of
    /// them.
    pub(crate) fn writes(&self) -> &[SaturatedWriteTiming; CHANNEL_SAMPLE_COUNT_USIZE] {
        &self.writes
    }
}

impl Serialize for SaturatedTimingBatch {
    /// Serialized as a flat JSON array of the per-write timings, in issue order. Hand-written
    /// because serde's blanket array impls stop at thirty-two elements, exactly as
    /// [`RawLatencies`](crate::observation::raw_latencies::RawLatencies) does for the same batch
    /// size — so the fixed-size array can stay the structural proof of the count.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.writes.iter())
    }
}
