//! One validated `(axis, rung)` pair — the scale an attempt holds fixed while it measures.

use anyhow::Result;
use serde::Serialize;

use crate::view_read_set_campaign::axis_ladder::{AxisLadder, LadderRungIndex};
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;

/// The validated `(axis, rung)` pair that names one scale point, per the spec's `ScalePoint`.
///
/// Every scale point is a separately provisioned fresh server, so this pair is part of the attempt's
/// durable identity rather than a position within a walk.
///
/// Both fields are private and there is no literal construction, no `new`, and no setter: the only
/// values in existence come from [`Self::validated`] or [`Self::ladder`]. Both obtain their rung
/// from the [`AxisLadder`] of the *same* axis they then store, so a rung minted against a longer
/// ladder cannot be paired with a shorter axis. That is what makes "an out-of-range or wrong-ladder
/// rung is unrepresentable" a property of the API surface rather than of caller discipline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct ScalePoint {
    axis: ExperimentAxis,
    rung: LadderRungIndex,
}

impl ScalePoint {
    /// Every scale point of `axis` in ascending ladder order — the whole frozen ladder as identities.
    ///
    /// Infallible because the rungs come from the ladder itself; this is the path the inventory uses,
    /// so the common case cannot fail and therefore cannot be handled wrongly.
    pub(crate) fn ladder(axis: ExperimentAxis) -> Vec<Self> {
        let ladder = AxisLadder::of(axis);
        ladder
            .rungs()
            .into_iter()
            .map(|rung| Self { axis, rung })
            .collect()
    }

    /// The scale point at `position` on `axis`'s ladder, failing loud when that ladder has no such
    /// rung.
    ///
    /// The fallible path for positions arriving from outside the program — a replayed ledger line, a
    /// CLI argument — where range membership is a claim to check rather than a property already
    /// established by construction.
    pub(crate) fn validated(axis: ExperimentAxis, position: usize) -> Result<Self> {
        let rung = AxisLadder::of(axis).validated(position)?;
        Ok(Self { axis, rung })
    }

    /// Which frozen finite range this scale point is drawn from.
    pub(crate) fn axis(self) -> ExperimentAxis {
        self.axis
    }

    /// This scale point's 0-based position on its axis ladder.
    pub(crate) fn rung(self) -> LadderRungIndex {
        self.rung
    }

    /// The frozen quantity held at exactly this value for the whole of an attempt's measurement.
    ///
    /// Reads the ladder of the axis stored alongside the rung, so the value returned is always the
    /// one this scale point's own ladder declares.
    pub(crate) fn scale(self) -> u64 {
        AxisLadder::of(self.axis).scale_at(self.rung)
    }
}

#[cfg(test)]
mod tests;
