//! Why the admitted replicates do not form the pair §569 requires.

use std::collections::BTreeSet;
use std::fmt;

/// The typed reason a set of admitted replicates is not an analysable pair.
///
/// Separate from [`AdmissionRefusal`](super::admission_refusal::AdmissionRefusal) because the two
/// answer different questions: that one is about a single record's own completeness, this one is
/// about the set. A ledger can contain two individually perfect records that still cannot be paired,
/// and collapsing both into one error type would hide which of those happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PairRefusal {
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
