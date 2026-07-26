//! The 0-based position of one rung within its axis ladder.

use serde::Serialize;

/// The 0-based position of a rung in the ladder of the axis it belongs to.
///
/// Declared *inside* [`AxisLadder`](super::AxisLadder)'s own module rather than beside it, so
/// [`Self::at`]'s `pub(super)` resolves to that one module. `AxisLadder` is therefore the only
/// code in the crate that can mint a rung — structurally, not by convention — and it is the code
/// that knows how long each ladder is.
///
/// Deliberately says nothing about *which* ladder it indexes; that is the axis, and the two travel
/// together only inside a [`ScalePoint`](crate::view_read_set_campaign::scale_point::ScalePoint).
/// A bare rung index therefore never names a scale on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct LadderRungIndex(u8);

impl LadderRungIndex {
    /// Mint the rung at `position`.
    ///
    /// Visible only within [`super`] — the module that declares `AxisLadder` — so every rung in
    /// existence has passed through a length check against its own axis's ladder. No sibling module
    /// of `axis_ladder` can reach this.
    pub(super) const fn at(position: u8) -> Self {
        Self(position)
    }

    /// The 0-based rung number.
    pub(crate) fn get(self) -> usize {
        self.0 as usize
    }
}
