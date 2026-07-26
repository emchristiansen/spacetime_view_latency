//! The frozen finite ladder one axis is swept over.
//!
//! **Module topology is the enforcement mechanism here, not a comment.** Both types live in the
//! private, *childless* inline module [`sealed`], and [`LadderRungIndex`]'s field is private to it.
//! Rust makes a private field visible to its declaring module **and every descendant**, so a type
//! whose minter sits in the same file as other modules is only as confined as that file's module
//! tree: a `#[cfg(test)] mod tests` child, or any child added later, would be able to construct one
//! directly and bypass the length check. Giving `sealed` no children makes the set of code that can
//! mint a rung exactly the set that reads a ladder length first — enforced by the compiler, and
//! stable under anything added to this file afterwards.
//!
//! Everything else in this file — the tests, and anything future — is a **sibling** of `sealed`, not
//! a descendant, so it sees only the re-exported public surface.

mod sealed {
    use anyhow::{ensure, Result};
    use serde::Serialize;

    use crate::view_read_set_campaign::campaign_params::{
        UNRELATED_GLOBAL_ROWS_LADDER, UNRELATED_GLOBAL_ROWS_LADDER_LEN,
    };
    use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;

    /// One axis's frozen finite range, and the only code in the crate that can mint a rung against
    /// it.
    ///
    /// This is the spec's "axis ladder validator". Every [`LadderRungIndex`] in existence was
    /// produced by [`Self::rungs`] or [`Self::validated`], both of which read the length of *this*
    /// axis's ladder first — and no code outside this childless module can produce one at all, so
    /// out-of-range construction is absent from the API surface rather than rejected at runtime.
    ///
    /// Carries no ladder data of its own — it is a typed view onto the frozen constants — so a
    /// rung's scale value can never drift from the preregistered literal.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct AxisLadder {
        axis: ExperimentAxis,
    }

    /// The 0-based position of a rung in the ladder of the axis it belongs to.
    ///
    /// Proves range validity only, and structurally: the field is private to this childless module
    /// and there is no constructor, so the only expressions in the crate that can build one are the
    /// two struct literals below.
    ///
    /// Deliberately says nothing about *which* ladder it indexes; that is the axis, and the two
    /// travel together only inside a
    /// [`ScalePoint`](crate::view_read_set_campaign::scale_point::ScalePoint). A bare rung index
    /// therefore never names a scale on its own.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    #[serde(transparent)]
    pub(crate) struct LadderRungIndex(u8);

    impl LadderRungIndex {
        /// The 0-based rung number.
        pub(crate) fn get(self) -> usize {
            self.0 as usize
        }
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
                    LadderRungIndex(position)
                })
                .collect()
        }

        /// Mint the rung at `position`, failing loud when this axis's ladder has no such rung.
        ///
        /// The fallible path exists for values arriving from outside the program — a replayed
        /// ledger, a CLI argument — where range membership is a claim to check rather than a
        /// property already established by construction.
        pub(crate) fn validated(self, position: usize) -> Result<LadderRungIndex> {
            let len = self.len();
            ensure!(
                position < len,
                "rung {position} is outside the {len}-rung ladder of {:?}",
                self.axis,
            );
            let position =
                u8::try_from(position).expect("a position below the ladder length fits u8");
            Ok(LadderRungIndex(position))
        }

        /// The frozen scale value held at `rung` for the whole of an attempt's measurement.
        ///
        /// Total: every [`LadderRungIndex`] came from this module against some axis's ladder. The
        /// one remaining way to pair a rung with the wrong axis is to call this with a rung minted
        /// from a longer ladder, which
        /// [`ScalePoint`](crate::view_read_set_campaign::scale_point::ScalePoint) makes unreachable
        /// by binding the two together at construction.
        pub(crate) fn scale_at(self, rung: LadderRungIndex) -> u64 {
            self.scales()[rung.get()]
        }
    }
}

pub(crate) use sealed::{AxisLadder, LadderRungIndex};

#[cfg(test)]
mod tests;
