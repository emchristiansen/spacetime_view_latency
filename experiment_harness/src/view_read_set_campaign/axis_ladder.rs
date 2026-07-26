//! The frozen finite ladder one axis is swept over.

use anyhow::{ensure, Result};

use crate::view_read_set_campaign::campaign_params::{
    UNRELATED_GLOBAL_ROWS_LADDER, UNRELATED_GLOBAL_ROWS_LADDER_LEN,
};
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;

mod ladder_rung_index;

pub(crate) use ladder_rung_index::LadderRungIndex;

/// One axis's frozen finite range, and the only code in the crate that can mint a rung against it.
///
/// This is the spec's "axis ladder validator". [`LadderRungIndex`] is declared in this module's own
/// private child, so its constructor's `pub(super)` reaches exactly here: every rung in existence
/// was produced by [`Self::rungs`] or [`Self::validated`], both of which read the length of *this*
/// axis's ladder. Out-of-range construction is therefore absent from the API surface rather than
/// rejected at runtime.
///
/// Carries no ladder data of its own — it is a typed view onto the frozen constants — so a rung's
/// scale value can never drift from the preregistered literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AxisLadder {
    axis: ExperimentAxis,
}

impl AxisLadder {
    /// The frozen ladder of `axis`.
    pub(crate) const fn of(axis: ExperimentAxis) -> Self {
        Self { axis }
    }

    /// The axis this ladder belongs to.
    pub(crate) fn axis(self) -> ExperimentAxis {
        self.axis
    }

    /// The frozen scale value at each rung, in ascending ladder order.
    pub(crate) fn scales(self) -> &'static [u64] {
        match self.axis {
            ExperimentAxis::UnrelatedGlobalRows => &UNRELATED_GLOBAL_ROWS_LADDER,
        }
    }

    /// How many rungs this ladder has — the spec's `r`.
    pub(crate) fn len(self) -> usize {
        match self.axis {
            ExperimentAxis::UnrelatedGlobalRows => UNRELATED_GLOBAL_ROWS_LADDER_LEN,
        }
    }

    /// Every rung of this ladder in ascending order — the only rung values in existence for this
    /// axis. Consuming this in order is how a caller walks the ladder.
    pub(crate) fn rungs(self) -> Vec<LadderRungIndex> {
        (0..self.len())
            .map(|position| {
                let position = u8::try_from(position)
                    .expect("a frozen ladder is far shorter than u8::MAX rungs");
                LadderRungIndex::at(position)
            })
            .collect()
    }

    /// Mint the rung at `position`, failing loud when this axis's ladder has no such rung.
    ///
    /// The fallible path exists for values arriving from outside the program — a replayed ledger, a
    /// CLI argument — where range membership is a claim to check rather than a property already
    /// established by construction.
    pub(crate) fn validated(self, position: usize) -> Result<LadderRungIndex> {
        let len = self.len();
        ensure!(
            position < len,
            "rung {position} is outside the {len}-rung ladder of {:?}",
            self.axis,
        );
        let position = u8::try_from(position).expect("a position below the ladder length fits u8");
        Ok(LadderRungIndex::at(position))
    }

    /// The frozen scale value held at `rung` for the whole of an attempt's measurement.
    ///
    /// Total: every [`LadderRungIndex`] came from this module against some axis's ladder. The one
    /// remaining way to pair a rung with the wrong axis is to call this with a rung minted from a
    /// longer ladder, which [`ScalePoint`](crate::view_read_set_campaign::scale_point::ScalePoint)
    /// makes unreachable by binding the two together at construction.
    pub(crate) fn scale_at(self, rung: LadderRungIndex) -> u64 {
        self.scales()[rung.get()]
    }
}

#[cfg(test)]
mod tests;
