//! A purpose-built shared counter for one dose's SDK insert/delete/update delivery events.

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use super::dose_event_counts::DoseEventCounts;

/// The shared tally of the SDK's client-visible insert/delete/update events for the dose currently in
/// flight. Registered SDK row callbacks increment it as events are delivered; the run driver reads and
/// clears it once per dose at a proven quiescent boundary and hands the drained counts to
/// [`EventEvidence::checked`](super::event_evidence::EventEvidence::checked).
///
/// **Why atomics, not a mutex.** The three counts are independent monotone tallies incremented from the
/// SDK's message-processing callbacks and read from the driver thread; each increment is a single
/// [`AtomicU64::fetch_add`], so no lock is needed to keep them consistent and no `std::Mutex` is
/// introduced.
///
/// **Why [`Ordering::Relaxed`] suffices for cross-thread visibility.** On the SDK's single
/// message-processing thread the row callbacks for a confirmed write are invoked *before* that write's
/// reducer confirmation is sent over the confirmed-read mpsc channel (proven from the pinned SDK 2.6.1
/// source: `db_connection.rs` applies the update and invokes row callbacks around line 312, then the
/// reducer confirmation callback runs around line 219). The driver reads this counter only after that
/// channel receive completes, and the channel receive establishes happens-before with the send, so every
/// prior callback increment is visible without these atomics carrying the ordering themselves. `Relaxed`
/// therefore gives all that is needed here: atomic per-increment accumulation.
///
/// **Why [`Self::drain`] still requires no callbacks in flight.** The three swaps are three separate
/// atomic operations, not one atomic group, so an increment landing between the insert swap and the
/// update swap could be split across two drains. The happens-before above only bounds *visibility*, not
/// concurrency, so `drain` is sound only when called at the per-dose quiescent boundary — after the
/// dose's confirmations have arrived and before the next dose's callbacks can fire.
pub(crate) struct DoseEventCounter {
    inserts: AtomicU64,
    deletes: AtomicU64,
    updates: AtomicU64,
}

impl DoseEventCounter {
    /// A fresh counter with every tally at zero.
    pub(crate) fn new() -> Self {
        Self {
            inserts: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
            updates: AtomicU64::new(0),
        }
    }

    /// Record one delivered insert event. Called from the SDK's insert row callback.
    pub(crate) fn record_insert(&self) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record one delivered delete event. Called from the SDK's delete row callback.
    pub(crate) fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record one delivered update event. Called from the SDK's update row callback.
    pub(crate) fn record_update(&self) {
        self.updates.fetch_add(1, Ordering::Relaxed);
    }

    /// Read the accumulated counts and reset every tally to zero, returning the drained
    /// [`DoseEventCounts`]. Each tally is cleared with an atomic [`AtomicU64::swap`]. The three swaps are
    /// not one atomic group, so this is sound only at the per-dose quiescent boundary described on the
    /// type: with no callbacks in flight, the drained counts belong to exactly one dose and the next dose
    /// starts from zero.
    pub(crate) fn drain(&self) -> DoseEventCounts {
        DoseEventCounts::new(
            self.inserts.swap(0, Ordering::Relaxed),
            self.deletes.swap(0, Ordering::Relaxed),
            self.updates.swap(0, Ordering::Relaxed),
        )
    }
}
