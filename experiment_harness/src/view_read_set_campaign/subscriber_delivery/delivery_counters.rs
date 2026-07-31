//! The counter handles one measured window's row callbacks are registered with.

use std::sync::Arc;

use crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counter::SubscriberRowCounter;

/// One handle on an attempt's row counter per callback the SDK is about to register.
///
/// Prebuilt rather than cloned at registration because the reconnect registers these *inside* its
/// measured interval — the connection they attach to does not exist before it starts. Cloning here
/// is the whole of what can be moved out of that interval; the SDK's own per-callback boxing cannot.
///
/// Named fields rather than three positional `Arc`s of one type: a swap would register the insert
/// callback against the update tally, misreporting the ledger with nothing able to detect it.
pub(crate) struct DeliveryCounters {
    pub(crate) inserts: Arc<SubscriberRowCounter>,
    pub(crate) updates: Arc<SubscriberRowCounter>,
    pub(crate) deletes: Arc<SubscriberRowCounter>,
}

impl DeliveryCounters {
    /// Clone one handle per callback from the attempt's counter.
    pub(crate) fn of(counter: &Arc<SubscriberRowCounter>) -> Self {
        Self {
            inserts: counter.clone(),
            updates: counter.clone(),
            deletes: counter.clone(),
        }
    }
}
