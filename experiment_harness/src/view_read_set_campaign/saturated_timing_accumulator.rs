//! Accumulates one issue-and-confirmation offset pair per saturated write, by issue index, into a
//! [`SaturatedTimingBatch`].

use anyhow::{anyhow, ensure, Result};

use crate::view_read_set_campaign::campaign_params::{
    CHANNEL_SAMPLE_COUNT, CHANNEL_SAMPLE_COUNT_USIZE,
};
use crate::view_read_set_campaign::saturated_timing_batch::SaturatedTimingBatch;
use crate::view_read_set_campaign::saturated_write_timing::SaturatedWriteTiming;

/// Collects the saturated batch's confirmed timings — one per write, *addressed by its issue index*
/// — then seals them into an issue-ordered [`SaturatedTimingBatch`].
///
/// Copies [`DoseLatencyAccumulator`](crate::observation::dose_latency_accumulator) over this
/// campaign's timing pair and batch length. Slot addressing matters more here than there: this
/// channel issues every write before awaiting any, and its estimand is the Theil–Sen slope of
/// latency against *issue index*, so a batch sealed in completion order would reduce to a different
/// number while looking identical. [`Self::record`] rejects an out-of-range or already-filled slot;
/// [`Self::seal`] rejects any empty one.
///
/// Proves cardinality, single-fill, and issue order — not that the timings are genuine, which is the
/// measurement transition's to establish.
pub(crate) struct SaturatedTimingAccumulator {
    /// One slot per issue index; `Some` once that write's confirmation has been recorded.
    slots: Vec<Option<SaturatedWriteTiming>>,
    /// The number of filled slots, so completion is a count check rather than a full scan.
    filled: usize,
}

impl SaturatedTimingAccumulator {
    /// A fresh accumulator with one empty slot per write of a full saturated batch.
    pub(crate) fn new() -> Self {
        Self {
            slots: (0..CHANNEL_SAMPLE_COUNT_USIZE).map(|_| None).collect(),
            filled: 0,
        }
    }

    /// Record one confirmed write's timing pair at its issue `index`. Fails loud if `index` is out
    /// of range for the batch, or if that slot was already recorded — neither is representable in a
    /// well-formed batch.
    pub(crate) fn record(&mut self, index: usize, timing: SaturatedWriteTiming) -> Result<()> {
        let slot = self.slots.get_mut(index).ok_or_else(|| {
            anyhow!(
                "confirmed write index {index} is out of range for a {CHANNEL_SAMPLE_COUNT}-write \
                 saturated batch"
            )
        })?;
        ensure!(
            slot.is_none(),
            "duplicate confirmation for saturated write index {index}"
        );
        *slot = Some(timing);
        self.filled += 1;
        Ok(())
    }

    /// Whether every write's confirmation has been recorded — the barrier's completion condition.
    pub(crate) fn is_complete(&self) -> bool {
        self.filled == CHANNEL_SAMPLE_COUNT_USIZE
    }

    /// Seal the batch into an issue-ordered [`SaturatedTimingBatch`], failing loud if any write's
    /// slot is still empty. [`SaturatedTimingBatch::sealed`] re-checks the cardinality against its
    /// fixed-size array.
    pub(crate) fn seal(self) -> Result<SaturatedTimingBatch> {
        let mut writes = Vec::with_capacity(CHANNEL_SAMPLE_COUNT_USIZE);
        for (index, slot) in self.slots.into_iter().enumerate() {
            let timing = slot
                .ok_or_else(|| anyhow!("missing confirmation for saturated write index {index}"))?;
            writes.push(timing);
        }
        SaturatedTimingBatch::sealed(writes)
    }
}
