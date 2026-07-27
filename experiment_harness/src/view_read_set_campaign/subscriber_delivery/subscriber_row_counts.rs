//! The insert/update/delete counts read off one attempt's row counter.

/// The three delivery counts observed over one attempt's measured window — a plain value snapshot
/// with no shared state, the counterpart of
/// [`DoseEventCounts`](crate::observation::dose_event_counts::DoseEventCounts) for this campaign.
///
/// Named fields written by exactly one producer,
/// [`SubscriberRowCounter::snapshot`](super::subscriber_row_counter::SubscriberRowCounter::snapshot),
/// rather than a positional constructor over three `u64`s: a swap there would relabel deliveries in
/// the ledger permanently and nothing downstream could detect it. The visibility is the same
/// module-wide trust boundary `DoseEventCounts` draws around `crate::observation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SubscriberRowCounts {
    /// Rows the SDK delivered as inserts, including the cold subscription's initial snapshot.
    pub(in crate::view_read_set_campaign::subscriber_delivery) inserts: u64,
    /// Rows the SDK delivered as in-place updates — what the measured mutation produces.
    pub(in crate::view_read_set_campaign::subscriber_delivery) updates: u64,
    /// Rows the SDK delivered as deletes.
    pub(in crate::view_read_set_campaign::subscriber_delivery) deletes: u64,
}

impl SubscriberRowCounts {
    /// Insert events delivered over the window.
    pub(crate) fn inserts(self) -> u64 {
        self.inserts
    }

    /// Update events delivered over the window.
    pub(crate) fn updates(self) -> u64 {
        self.updates
    }

    /// Delete events delivered over the window.
    pub(crate) fn deletes(self) -> u64 {
        self.deletes
    }
}
