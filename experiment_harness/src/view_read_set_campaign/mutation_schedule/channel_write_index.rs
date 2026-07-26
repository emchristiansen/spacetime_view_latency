//! The 0-based position of one measured write within its channel's batch.

use serde::Serialize;

/// The 0-based index of a measured write inside the frozen batch of exactly
/// [`CHANNEL_SAMPLE_COUNT`](crate::view_read_set_campaign::campaign_params::CHANNEL_SAMPLE_COUNT)
/// writes.
///
/// Declared *inside* [`MutationSchedule`](super::MutationSchedule)'s own module rather than beside
/// it, so [`Self::at`]'s `pub(super)` resolves to that one module. The schedule is therefore the
/// only code in the crate that can mint a write index — structurally, not by convention — and it is
/// the code that knows how long a batch is. Copies the visibility arrangement of
/// [`LadderRungIndex`](crate::view_read_set_campaign::axis_ladder::LadderRungIndex), for the same
/// reason.
///
/// A bare `u64` would let a caller ask for the target key or payload of write 5,000 — a write the
/// protocol never issues — and get a confident answer derived from a modulus that happily accepts
/// it. That answer would be wrong in the worst way: silently well-formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct ChannelWriteIndex(u64);

impl ChannelWriteIndex {
    /// Mint the write at `index`.
    ///
    /// Visible only within [`super`] — the module that declares `MutationSchedule` — so every write
    /// index in existence came from a walk of the frozen batch length.
    pub(super) const fn at(index: u64) -> Self {
        Self(index)
    }

    /// The 0-based write number, for the derivations that read it.
    pub(crate) fn get(self) -> u64 {
        self.0
    }
}
