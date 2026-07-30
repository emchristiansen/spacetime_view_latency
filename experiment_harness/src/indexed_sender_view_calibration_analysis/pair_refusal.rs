//! Why the admitted replicates do not form the pair §569 requires.

use std::collections::BTreeSet;
use std::fmt;

use crate::indexed_sender_view_calibration_analysis::refused_line::RefusedLine;

/// The typed reason a ledger is not an analysable pair.
///
/// Separate from [`AdmissionRefusal`](super::admission_refusal::AdmissionRefusal) because the two
/// answer different questions: that one is about a single record's own completeness, this one is
/// about the ledger. A ledger can contain two individually perfect records that still cannot be
/// paired, and collapsing both into one error type would hide which of those happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PairRefusal {
    /// Some line in the ledger was not admissible, so the ledger is not exactly the frozen
    /// inventory — whatever its other lines contain.
    ///
    /// **Checked before the ordinals, because it is the more general fault.** The freeze contains
    /// exactly two original records and nothing else. A ledger holding two perfect originals *plus*
    /// a failed attempt, a `Control` line, or a retry-shaped duplicate is therefore not that
    /// inventory, and analysing its two good lines would report a §569 pair drawn from an artifact
    /// whose extra content nobody looked at. The extra line may be the most informative thing in the
    /// file — a second attempt that failed says something about the method that two successes do
    /// not.
    ///
    /// Every refused line is carried with its position, so the report says *which* lines and *why*
    /// rather than only that some line failed.
    LedgerHasRefusedLines { refused: Vec<RefusedLine> },
    /// The admitted ordinals are not exactly the set the frozen inventory declares.
    ///
    /// Covers every way that can happen at once — too few, too many, a duplicate, or a foreign
    /// ordinal — because they are all one fact: the ordinals present are not the ordinals frozen.
    /// Both sets are carried so the report names which slots were found and which were expected
    /// rather than only that they differed.
    OrdinalsNotFrozenInventory {
        found: BTreeSet<u32>,
        expected: BTreeSet<u32>,
    },
    /// Two admitted records name the same replicate ordinal, so they are one attempt counted twice.
    ///
    /// Kept distinct from the set mismatch above because a duplicate is a *different* fault with a
    /// different remedy: set equality alone cannot see it, since `{0, 0}` and `{0}` collapse to the
    /// same set and would otherwise be reported as a missing replicate 1.
    DuplicateOrdinal { replicate: u32 },
}

impl fmt::Display for PairRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PairRefusal::LedgerHasRefusedLines { refused } => {
                write!(
                    f,
                    "the frozen inventory is exactly two original records, and this ledger contains \
                     {} line(s) that are not complete replicates, so it is not that inventory and \
                     no candidate count can be evaluated against it",
                    refused.len()
                )?;
                for line in refused {
                    write!(f, "\n  {line}")?;
                }
                Ok(())
            }
            PairRefusal::OrdinalsNotFrozenInventory { found, expected } => write!(
                f,
                "the decision rule is stated over both complete retained series of the frozen \
                 inventory, whose replicate ordinals are {expected:?}; the admitted records cover \
                 {found:?}. With either original absent the method is redesigned or deferred rather \
                 than analysed from one series, and a record at an ordinal the inventory never \
                 declared is not an original at all"
            ),
            PairRefusal::DuplicateOrdinal { replicate } => write!(
                f,
                "two admitted records both name replicate {replicate}, so they are one attempt \
                 counted twice rather than two independent ones"
            ),
        }
    }
}
