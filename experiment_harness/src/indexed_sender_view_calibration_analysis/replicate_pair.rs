//! The two complete replicates §569 evaluates every candidate `W` against.

use std::collections::BTreeSet;

use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::frozen_replicate_ordinals::frozen_replicate_ordinals;
use crate::indexed_sender_view_calibration_analysis::ledger_admission::LedgerAdmission;
use crate::indexed_sender_view_calibration_analysis::pair_refusal::PairRefusal;

/// The frozen inventory's two complete replicates, one per declared ordinal.
///
/// **This type is why "analysis from one series" is unrepresentable rather than merely forbidden.**
/// §569 evaluates each candidate `W` against *both complete retained series*, and directs that if
/// either attempt is absent the method is redesigned or deferred rather than analysed thinner. That
/// instruction is structural here: [`Self::of`] is the only constructor and the diagnostics take a
/// `ReplicatePair`, so a one-series report has no value to be built from.
///
/// **The requirement is the whole ledger, not merely two good lines in it.** [`Self::of`] takes a
/// [`LedgerAdmission`] — the outcome of offering every line of an actual ledger *file* — so a file
/// containing two perfect originals *plus* anything else cannot produce a pair. The freeze declares
/// exactly two records; a third line means the artifact is not that freeze, and reporting from its
/// good two would describe a pair while silently discarding whatever the extra line said.
///
/// **Then the frozen ordinal *set*, not two distinct ordinals.** Distinctness alone would accept
/// records at ordinals 2 and 3 — two perfectly distinct attempts that the inventory never declared —
/// and quietly promote them into "the two originals". The check is therefore set equality against
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
    /// Pair a whole ledger's admission outcome, refusing anything that is not exactly the frozen
    /// inventory.
    ///
    /// Takes the whole outcome rather than two arguments, so "a line was refused", "there were
    /// three", "there was one", "one was counted twice", and "one is not a declared slot" are
    /// refusals this type states rather than conditions a caller must remember to check before
    /// picking two.
    ///
    /// The three checks run from most general to most specific. Any refused line means the artifact
    /// is not the freeze at all, so that is decided first — by the admission itself, whose only exit
    /// yields the complete replicates *or* the refusal, never both. Duplicates come before set
    /// equality, so one attempt counted twice is reported as the duplicate it is rather than as a
    /// missing original.
    pub(crate) fn of(admission: LedgerAdmission) -> Result<Self, PairRefusal> {
        // Any refused line at all: the freeze is exactly two originals, so a ledger with a third
        // record — however that record failed — is not the inventory the rule is stated over. This
        // is not a check performed here but a condition of obtaining the replicates at all.
        let admitted = admission.into_complete_inventory()?;

        // Duplicates next: `{0, 0}` and `{0}` collapse to the same set, so a later set comparison
        // would report one attempt counted twice as a *missing* original.
        let mut found = BTreeSet::new();
        for replicate in &admitted {
            if !found.insert(replicate.replicate()) {
                return Err(PairRefusal::DuplicateOrdinal {
                    replicate: replicate.replicate(),
                });
            }
        }

        let expected = frozen_replicate_ordinals();
        if found != expected {
            return Err(PairRefusal::OrdinalsNotFrozenInventory { found, expected });
        }

        let mut ascending = admitted;
        ascending.sort_by_key(CompleteReplicate::replicate);
        let mut ascending = ascending.into_iter();
        // Set equality against the frozen inventory has already fixed the count, so both are present.
        let first = ascending
            .next()
            .expect("set equality with the frozen inventory guarantees both ordinals are present");
        let second = ascending
            .next()
            .expect("set equality with the frozen inventory guarantees both ordinals are present");
        Ok(Self { first, second })
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
