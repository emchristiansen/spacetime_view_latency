//! Accumulates one confirmed round-trip sample per measured write, by issue index, into a
//! [`RawLatencies`].

use anyhow::{anyhow, ensure, Result};

use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::{BATCH_SIZE, BATCH_SIZE_USIZE};

/// Collects the confirmed round-trip samples of one dose batch — one per write, *addressed by its
/// issue index* — then seals them into an issue-ordered [`RawLatencies`].
///
/// The slot addressing is what makes a missing or duplicated confirmation loud rather than silent.
/// A `Vec<LatencySample>` grown by a bare `push` would accept the samples in *completion* order and
/// could seal a batch in which one write's confirmation is missing while another's is counted twice
/// — the total is still `BATCH_SIZE`, but the vector is wrong and out of issue order. Here each of
/// the `BATCH_SIZE` slots is filled exactly once:
///
/// - [`Self::record`] fails loud if the index is out of range or its slot is already filled (a
///   duplicate confirmation), so a double-fire cannot overwrite or inflate the count.
/// - [`Self::seal`] fails loud if any slot is still empty (a missing confirmation), and emits the
///   samples in issue order `0..BATCH_SIZE`.
///
/// This type proves *cardinality, single-fill, and issue order* at the slot boundary; it
/// deliberately does **not** claim to prove *provenance*. That every recorded sample is a genuine
/// confirmed round trip is established by the private measurement transition that owns the
/// accumulator and records only measured latencies — not by the visibility of [`Self::record`]. The
/// honest boundary: the type bounds the shape, the measurement transition bounds the meaning.
pub(crate) struct DoseLatencyAccumulator {
    /// One slot per issue index; `Some` once that write's confirmation has been recorded.
    slots: Vec<Option<LatencySample>>,
    /// The number of filled slots, so completion is a count check rather than a full scan.
    filled: usize,
}

impl DoseLatencyAccumulator {
    /// A fresh accumulator with one empty slot per write of a full dose batch.
    pub(crate) fn new() -> Self {
        Self {
            slots: (0..BATCH_SIZE_USIZE).map(|_| None).collect(),
            filled: 0,
        }
    }

    /// Record one confirmed write's round-trip latency at its issue `index`. Fails loud if `index`
    /// is out of range for the batch, or if that slot was already recorded (a duplicate
    /// confirmation) — neither is representable in a well-formed batch.
    pub(crate) fn record(&mut self, index: usize, sample: LatencySample) -> Result<()> {
        let slot = self.slots.get_mut(index).ok_or_else(|| {
            anyhow!("confirmed write index {index} is out of range for a {BATCH_SIZE}-write dose")
        })?;
        ensure!(
            slot.is_none(),
            "duplicate confirmation for measured write index {index}"
        );
        *slot = Some(sample);
        self.filled += 1;
        Ok(())
    }

    /// Whether every write's confirmation has been recorded — the barrier's completion condition.
    pub(crate) fn is_complete(&self) -> bool {
        self.filled == BATCH_SIZE_USIZE
    }

    /// Seal the batch into an issue-ordered [`RawLatencies`], failing loud if any write's slot is
    /// still empty (a missing confirmation). The samples are emitted in issue order `0..BATCH_SIZE`,
    /// and [`RawLatencies::sealed`] re-checks the cardinality.
    pub(crate) fn seal(self) -> Result<RawLatencies> {
        let mut samples = Vec::with_capacity(BATCH_SIZE_USIZE);
        for (index, slot) in self.slots.into_iter().enumerate() {
            let sample = slot
                .ok_or_else(|| anyhow!("missing confirmation for measured write index {index}"))?;
            samples.push(sample);
        }
        RawLatencies::sealed(samples)
    }
}

#[cfg(test)]
mod tests;
