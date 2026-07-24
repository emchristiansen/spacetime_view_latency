//! The insert/delete/update counts drained from a dose's event counter.

/// The insert/delete/update counts drained from a
/// [`DoseEventCounter`](super::dose_event_counter::DoseEventCounter) for one dose — a plain value
/// snapshot with no shared state, fed straight into
/// [`EventEvidence::checked`](super::event_evidence::EventEvidence::checked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DoseEventCounts {
    inserts: u64,
    deletes: u64,
    updates: u64,
}

impl DoseEventCounts {
    /// Bundle one dose's drained event counts. `pub(in crate::observation)` so only a
    /// [`DoseEventCounter`](super::dose_event_counter::DoseEventCounter) drain mints one.
    pub(in crate::observation) fn new(inserts: u64, deletes: u64, updates: u64) -> Self {
        Self {
            inserts,
            deletes,
            updates,
        }
    }

    /// The number of insert events delivered this dose.
    pub(crate) fn inserts(self) -> u64 {
        self.inserts
    }

    /// The number of delete events delivered this dose.
    pub(crate) fn deletes(self) -> u64 {
        self.deletes
    }

    /// The number of update events delivered this dose.
    pub(crate) fn updates(self) -> u64 {
        self.updates
    }
}
