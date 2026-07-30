//! The two complete replicates §569 evaluates every candidate `W` against.

use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::pair_refusal::PairRefusal;

/// The frozen inventory's two complete replicates, one per declared ordinal.
///
/// **This type is why "analysis from one series" is unrepresentable rather than merely forbidden.**
/// §569 evaluates each candidate `W` against *both complete retained series*, and directs that if
/// either attempt is absent the method is redesigned or deferred rather than analysed thinner. That
/// instruction is structural here: [`Self::of`] is the only constructor and the diagnostics take a
/// `ReplicatePair`, so a one-series report has no value to be built from.
///
/// **The requirement is the frozen ordinal *set*, not two distinct ordinals.** Distinctness alone
/// would accept records at ordinals 2 and 3 — two perfectly distinct attempts that the inventory
/// never declared — and quietly promote them into "the two originals". The check is therefore set
/// equality against
/// [`frozen_replicate_ordinals`](super::frozen_replicate_ordinals::frozen_replicate_ordinals),
/// derived from the pilot's own `CalibrationReplicate::ALL`.
///
/// A duplicate is reported separately rather than folded into that comparison, because `{0, 0}` and
/// `{0}` are the same set: without its own check, one attempt counted twice would be reported as a
/// missing replicate. Counting it twice would make between-replicate disagreement — one of the four
/// things the decision rule weighs — read as identically zero.
#[derive(Debug, Clone)]
pub(crate) struct ReplicatePair {
    first: CompleteReplicate,
    second: CompleteReplicate,
}

impl ReplicatePair {
    /// Pair the admitted replicates, refusing anything that is not exactly the frozen inventory.
    ///
    /// Takes the whole admitted set rather than two arguments, so "there were three", "there was
    /// one", "one was counted twice", and "one is not a declared slot" are refusals this type states
    /// rather than conditions a caller must remember to check before picking two.
    ///
    /// Duplicates are detected before set equality, so one attempt counted twice is reported as the
    /// duplicate it is rather than as a missing original.
    pub(crate) fn of(admitted: Vec<CompleteReplicate>) -> Result<Self, PairRefusal> {
        todo!("duplicate detection, frozen-ordinal set equality, ascending pairing")
    }

    /// The lower-ordinal replicate.
    pub(crate) fn first(&self) -> &CompleteReplicate {
        &self.first
    }

    /// The higher-ordinal replicate.
    pub(crate) fn second(&self) -> &CompleteReplicate {
        &self.second
    }
}

#[cfg(test)]
mod tests;
