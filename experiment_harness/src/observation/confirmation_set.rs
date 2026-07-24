//! A slot-indexed presence set proving a fixed set of fallible confirmations each fired exactly once.

use anyhow::{anyhow, ensure, Result};

/// Tracks that each of a fixed number of expected confirmations — addressed by issue index — has
/// been observed exactly once. Unlike [`DoseLatencyAccumulator`](super::dose_latency_accumulator::DoseLatencyAccumulator)
/// it carries no payload; it only proves *presence and single-fire*, for confirmations whose success
/// contributes no latency sample (a Chronicle pair's prerequisite `message_visibility` insert, whose
/// job is only to land before its measured `chronicle_message` row).
///
/// The barrier must require the exact expected set, not merely the measured samples: a prerequisite
/// callback can fail, and if the barrier stopped as soon as every measured confirmation arrived, a
/// late or unread prerequisite failure would be silently dropped. Requiring every prerequisite
/// confirmation too means every prerequisite callback is awaited, so its failure is always observed.
///
/// An expected count of zero (the single-table message family, which registers no prerequisites) is
/// [`Self::is_complete`] immediately.
pub(crate) struct ConfirmationSet {
    /// One slot per expected confirmation; `true` once that index has been recorded.
    slots: Vec<bool>,
    /// The number of recorded slots, so completion is a count check rather than a scan.
    recorded: usize,
}

impl ConfirmationSet {
    /// A set expecting exactly `expected` confirmations, one per issue index `0..expected`.
    pub(crate) fn new(expected: usize) -> Self {
        Self {
            slots: vec![false; expected],
            recorded: 0,
        }
    }

    /// Record the confirmation at issue `index`. Fails loud if `index` is out of range for the
    /// expected set, or if that index was already recorded (a duplicate confirmation).
    pub(crate) fn record(&mut self, index: usize) -> Result<()> {
        let expected = self.slots.len();
        let slot = self.slots.get_mut(index).ok_or_else(|| {
            anyhow!(
                "confirmation index {index} is out of range for {expected} expected confirmations"
            )
        })?;
        ensure!(!*slot, "duplicate confirmation for index {index}");
        *slot = true;
        self.recorded += 1;
        Ok(())
    }

    /// Whether every expected confirmation has been recorded (vacuously true when none are expected).
    pub(crate) fn is_complete(&self) -> bool {
        self.recorded == self.slots.len()
    }
}

#[cfg(test)]
mod tests;
