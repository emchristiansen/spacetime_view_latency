//! The shared tally the SDK's row callbacks increment over one attempt's measured window.

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counts::SubscriberRowCounts;

/// One attempt's running count of the SDK's client-visible insert/update/delete deliveries.
///
/// Copies [`DoseEventCounter`](crate::observation::dose_event_counter::DoseEventCounter) — three
/// independent monotone tallies, one [`AtomicU64::fetch_add`] per delivery, so no lock is taken and
/// no `std::Mutex` appears — with its two rationales unchanged. [`Ordering::Relaxed`] suffices
/// because the SDK invokes a confirmed write's row callbacks on its message-processing thread before
/// that write's confirmation is sent over the harness's channel, and the channel receive the driver
/// waits on establishes the happens-before these atomics therefore need not carry.
///
/// **The one deviation, and why.** The Pilot's counter drains: it swaps every tally to zero because
/// its readings are per-dose and the next dose must start from zero. This campaign has one window
/// per attempt and reads it once, at the meter's close, so [`Self::snapshot`] *loads* instead. A
/// draining read would make a second observation of the same window report zeroes — fabricated
/// counts that would be indistinguishable from a connection that delivered nothing. Loading is also
/// why the three reads need no quiescent-boundary caveat about being split across two drains; they
/// are still read at the window's close, when the attempt's callbacks are done.
pub(crate) struct SubscriberRowCounter {
    inserts: AtomicU64,
    updates: AtomicU64,
    deletes: AtomicU64,
}

impl SubscriberRowCounter {
    /// A fresh counter with every tally at zero, opened before the cold subscription so that
    /// subscription's initial snapshot counts as delivered rows.
    pub(crate) fn new() -> Self {
        Self {
            inserts: AtomicU64::new(0),
            updates: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
        }
    }

    /// Record one delivered insert. Called from the SDK's insert row callback.
    pub(crate) fn record_insert(&self) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record one delivered in-place update. Called from the SDK's update row callback.
    pub(crate) fn record_update(&self) {
        self.updates.fetch_add(1, Ordering::Relaxed);
    }

    /// Record one delivered delete. Called from the SDK's delete row callback.
    pub(crate) fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    /// Read what this window has accumulated, leaving every tally where it stands.
    pub(crate) fn snapshot(&self) -> SubscriberRowCounts {
        SubscriberRowCounts {
            inserts: self.inserts.load(Ordering::Relaxed),
            updates: self.updates.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests;
