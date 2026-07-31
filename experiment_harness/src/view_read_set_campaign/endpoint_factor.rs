//! One block's endpoint factor `T = S_last / S_first` on one measurement channel.
//!
//! The type lives in the private, *childless* inline module [`sealed`]: a private field is visible
//! to its declaring module and every descendant, so a `tests` child could write `EndpointFactor(r)`
//! for a rational no ladder produced. The tests are `sealed`'s sibling instead.

mod sealed {
    use serde::{Serialize, Serializer};

    use crate::analysis::stats::rational::Rational;
    use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
    use crate::view_read_set_campaign::unrelated_global_rows_ladder_evidence::UnrelatedGlobalRowsLadderEvidence;

    /// The spec's `T`: one block's exact rational endpoint factor for one channel.
    ///
    /// The sole constructor takes a whole [`UnrelatedGlobalRowsLadderEvidence`], never a bare
    /// number, so a factor cannot exist without the six selected complete attempts that produced it.
    ///
    /// Construction is infallible because both failures are excluded by its input types: the ladder
    /// proves the six rungs are exactly the frozen ladder, and
    /// [`CellStatistic`](crate::view_read_set_campaign::cell_statistic::CellStatistic) proves every
    /// `S` is strictly positive, so `S_first` is never zero. `T` is therefore strictly positive too.
    ///
    /// `T` is compared against `5/4` and `F` by exact rational arithmetic. A factor like `10/3` has
    /// no exact binary representation, so a float on this path could move a value across a decision
    /// boundary; floating point stays display-only.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct EndpointFactor(Rational);

    impl EndpointFactor {
        /// Compute this ladder's endpoint factor on one channel.
        pub(crate) fn of(
            ladder: &UnrelatedGlobalRowsLadderEvidence,
            channel: MeasurementChannel,
        ) -> Self {
            Self(super::endpoint_ratio(super::ladder_cells(ladder, channel)))
        }

        /// The exact value, for the interval and classification arithmetic that consumes it.
        pub(crate) fn get(self) -> Rational {
            self.0
        }
    }

    impl Serialize for EndpointFactor {
        /// The exact `[numerator, denominator]` pair rather than a float, matching `CellStatistic`,
        /// so the ledger retains the value the classification actually used.
        fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
            serializer.collect_seq(super::exact_pair(self.0).iter())
        }
    }
}

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::axis_ladder::LadderRungIndex;
use crate::view_read_set_campaign::campaign_params::UNRELATED_GLOBAL_ROWS_LADDER_LEN;
use crate::view_read_set_campaign::cell_statistic::CellStatistic;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::unrelated_global_rows_ladder_evidence::UnrelatedGlobalRowsLadderEvidence;

// Exposed as a type alias rather than a `use` re-export because the classifier that will name this
// does not exist yet: an unused `use` is an `unused_imports` warning, which must never be silenced,
// whereas an alias preserves the type and its associated functions identically and an unexercised
// one is ordinary dead code, already governed crate-wide by the skeleton's `#![allow(dead_code)]`.
// It becomes a plain `use` once the interval layer names it, as the ladder evidence just did here.
pub(crate) type EndpointFactor = sealed::EndpointFactor;

/// Each rung's index paired with its statistic on one channel.
///
/// Factored out of [`sealed`] to keep that module to the invariant alone. This projection is
/// inspection-covered, not exercised by the tests: it takes the ladder itself, and
/// [`SelectedCompleteAttempt`](crate::view_read_set_campaign::reconciled_campaign::SelectedCompleteAttempt)
/// is mintable only from a live campaign. It is a chain of typed accessors with no decision in it —
/// the decision is [`endpoint_ratio`], which the tests do exercise.
fn ladder_cells(
    ladder: &UnrelatedGlobalRowsLadderEvidence,
    channel: MeasurementChannel,
) -> [(LadderRungIndex, CellStatistic); UNRELATED_GLOBAL_ROWS_LADDER_LEN] {
    ladder.rungs().each_ref().map(|selected| {
        (
            selected.key().scale().rung(),
            selected
                .artifact()
                .scale_point_evidence()
                .statistic(channel),
        )
    })
}

/// `T = S_last / S_first`, the endpoints taken in rung order.
///
/// The sort is load-bearing, not defensive: sealing a ladder validates the rung *multiset* and
/// stores the selections in the caller's order, so array position is not rung order. Reading
/// position would compute a ratio between two arbitrary rungs — still a plausible positive number,
/// and nothing downstream would reject it.
///
/// Only the endpoints enter `T`; the intermediate rungs are retained on the ladder for curves and
/// detection bounds, and the classification asserts nothing about the shape between them.
fn endpoint_ratio(
    mut cells: [(LadderRungIndex, CellStatistic); UNRELATED_GLOBAL_ROWS_LADDER_LEN],
) -> Rational {
    cells.sort_by_key(|(rung, _)| *rung);
    // Irrefutable: the array type pins six rungs, so a first and a last always exist.
    let [(_, first), .., (_, last)] = cells;
    last.get().div(first.get())
}

/// The reduced `[numerator, denominator]` pair a factor serializes as.
///
/// A sibling of [`sealed`] so the serialized content is testable without minting an
/// [`EndpointFactor`], which needs a live campaign's ladder.
fn exact_pair(value: Rational) -> [i128; 2] {
    [value.numerator(), value.denominator()]
}

// A *sibling* of `sealed`, never a child, so these tests cannot write the struct literal around a
// number no ladder produced.
#[cfg(test)]
mod tests;
