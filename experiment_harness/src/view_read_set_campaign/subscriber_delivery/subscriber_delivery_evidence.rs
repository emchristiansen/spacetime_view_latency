//! What one attempt's measured subscriber actually received, recorded once its window closed.
//!
//! **Module topology is the enforcement mechanism here, not a comment.** The record lives in the
//! private, *childless* inline module [`sealed`]. Rust makes a private field visible to its declaring
//! module **and every descendant**, so declaring this beside a `tests` child — or any child added
//! later — would let that module write the struct literal and state delivery figures no connection
//! ever produced. `sealed` has no children, so [`SubscriberDeliveryEvidence::closed`] is the only
//! door.
//!
//! **And that constructor cannot be opened without a live meter**, because it takes a
//! [`ClosedWindow`](super::subscriber_delivery_meter::ClosedWindow) that only a meter's close can
//! build. So this file has no `tests` module: there is nothing a test could pass, and adding one
//! that sidestepped the window would be the seam this arrangement exists to close.

mod sealed {
    use serde::Serialize;
    use spacetimedb_sdk::SubscriptionHandle as _;

    use crate::module_artifact::bindings::SubscriptionHandle;
    use crate::view_read_set_campaign::subscriber_delivery::subscriber_delivery_meter::ClosedWindow;

    /// The supporting delivery facts of one attempt's measured window.
    ///
    /// **Why the three row kinds stay separate.** They answer different questions: the cold
    /// subscription's initial snapshot arrives as inserts, the measured mutation is an in-place
    /// update, and a delete would mean the fixed cardinality the protocol holds constant did not
    /// hold. Summing them at the door would lose exactly the distinction that makes the record worth
    /// keeping, so only [`Self::delivered_rows`] adds them, and it derives the sum rather than
    /// storing a second copy that could disagree with the parts.
    ///
    /// **What each field is a fact about.** All six describe *the connection*, over the window
    /// between the meter's open and its close — not the candidate's read set, which is the
    /// composition finding's job. What "bytes" and "frames" mean, and why they cannot be divided
    /// into a per-row figure, is on
    /// [`subscriber_delivery`](crate::view_read_set_campaign::subscriber_delivery).
    ///
    /// **What it does not claim.** That the connection was the measured subscriber's, or that the
    /// window was the attempt's. Both are properties of the driver's measurement path, exactly as
    /// genuineness is for every other evidence type in this campaign. What sealing adds is that no
    /// figure here was stated by a caller: each was counted, read coherently, or derived at this
    /// constructor.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct SubscriberDeliveryEvidence {
        delivered_row_inserts: u64,
        delivered_row_updates: u64,
        delivered_row_deletes: u64,
        received_wire_bytes: u64,
        received_frames: u64,
        active_subscription_handles: u32,
    }

    impl SubscriberDeliveryEvidence {
        /// Record what a closed window measured.
        ///
        /// **The handles are the handles, never a count.** The active figure is derived here by
        /// asking each retained subscription whether it is still active. A `u32` parameter would
        /// have been a free input — any value accepted, nothing able to contradict it — which is the
        /// forgeability the composition finding shed at checkpoint `a598e310`.
        ///
        /// A handle no longer active at close is one that ended or errored during the window, so
        /// counting only the active ones is the honest figure: it says how many subscriptions were
        /// still delivering when measurement stopped.
        pub(in crate::view_read_set_campaign::subscriber_delivery) fn closed(
            window: ClosedWindow,
            handles: &[SubscriptionHandle],
        ) -> Self {
            let rows = window.rows();
            let wire = window.wire();
            let active = handles.iter().filter(|handle| handle.is_active()).count();
            Self {
                delivered_row_inserts: rows.inserts(),
                delivered_row_updates: rows.updates(),
                delivered_row_deletes: rows.deletes(),
                received_wire_bytes: wire.bytes(),
                received_frames: wire.frames(),
                active_subscription_handles: u32::try_from(active).expect(
                    "one attempt retains one subscription per measured role, so the count of \
                     active handles is far below the top of u32",
                ),
            }
        }

        /// Rows the SDK delivered as inserts, the cold subscription's initial snapshot included.
        pub(crate) fn delivered_row_inserts(&self) -> u64 {
            self.delivered_row_inserts
        }

        /// Rows the SDK delivered as in-place updates — what the measured mutation produces.
        pub(crate) fn delivered_row_updates(&self) -> u64 {
            self.delivered_row_updates
        }

        /// Rows the SDK delivered as deletes.
        pub(crate) fn delivered_row_deletes(&self) -> u64 {
            self.delivered_row_deletes
        }

        /// Compressed on-the-wire bytes the connection received over the window.
        pub(crate) fn received_wire_bytes(&self) -> u64 {
            self.received_wire_bytes
        }

        /// Websocket frames the connection received over the window, of every kind.
        pub(crate) fn received_frames(&self) -> u64 {
            self.received_frames
        }

        /// Retained subscriptions still reporting themselves active when the window closed.
        pub(crate) fn active_subscription_handles(&self) -> u32 {
            self.active_subscription_handles
        }

        /// Total rows delivered — **derived**, never stored.
        ///
        /// The three kinds partition the SDK's client-visible row events, so their sum is the whole
        /// of what was delivered. Deriving it keeps the parts authoritative: a stored total could
        /// disagree with them, and there would be no way to tell which was right.
        pub(crate) fn delivered_rows(&self) -> u64 {
            self.delivered_row_inserts
                .checked_add(self.delivered_row_updates)
                .and_then(|rows| rows.checked_add(self.delivered_row_deletes))
                .expect(
                    "each kind counts client-visible row events of one connection over one \
                     attempt's window, so their total is that window's own event count and cannot \
                     overflow u64",
                )
        }
    }
}

pub(crate) use sealed::SubscriberDeliveryEvidence;
