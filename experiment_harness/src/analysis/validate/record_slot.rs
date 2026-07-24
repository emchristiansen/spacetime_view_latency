//! Which record the seed-derived schedule grammar expects at a sequence position.

use crate::dataset::dose_index::DoseIndex;

/// The record the preregistered schedule grammar expects at a given campaign sequence position within a
/// run: the run's single manifest (at the run's first position), or the cumulative-dose observation for a
/// specific 1-based [`DoseIndex`]. Compared field-for-field against the decoded actual record to locate a
/// within-run ordering fault — a manifest that is not at its run's start, or a dose observation out of
/// canonical ladder order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecordSlot {
    /// The run's manifest record, expected at the run's first sequence position.
    Manifest,
    /// The cumulative-dose observation for this 1-based dose index, expected in canonical ladder order.
    Dose(DoseIndex),
}
