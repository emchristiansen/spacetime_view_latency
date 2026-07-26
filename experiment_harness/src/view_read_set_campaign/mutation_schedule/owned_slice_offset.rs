//! The 0-based position of one row within the measured identity's own slice.

use serde::Serialize;

/// The 0-based offset of a row inside the owned slice of exactly
/// [`SUBSCRIBER_VISIBLE_ROWS_BASELINE`](crate::view_read_set_campaign::campaign_params::SUBSCRIBER_VISIBLE_ROWS_BASELINE)
/// rows.
///
/// Declared inside [`MutationSchedule`](super::MutationSchedule)'s own module so [`Self::at`]'s
/// `pub(super)` reaches only there, exactly as
/// [`ChannelWriteIndex`](super::ChannelWriteIndex) does. Every offset in existence therefore came
/// from a walk of the frozen slice, and cannot name a key that was never seeded.
///
/// Distinct from a key: the offset is added to `OWNED_KEY_BASE` to reach an `entity_uuid`. Keeping
/// them separate types stops a raw key from being passed where a slice position is expected, which
/// on the frozen constants would silently land nine rows away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct OwnedSliceOffset(u64);

impl OwnedSliceOffset {
    /// Mint the offset at `offset`.
    ///
    /// Visible only within [`super`], so every offset has passed through a walk of the frozen owned
    /// slice.
    pub(super) const fn at(offset: u64) -> Self {
        Self(offset)
    }

    /// The 0-based offset within the owned slice.
    pub(crate) fn get(self) -> u64 {
        self.0
    }
}
