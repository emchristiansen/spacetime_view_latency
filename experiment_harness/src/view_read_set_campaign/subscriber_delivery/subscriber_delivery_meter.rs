//! The live meter whose close is the only way one attempt's delivery evidence comes into being.
//!
//! **Module topology is the enforcement mechanism here, not a comment.** The meter and its closed
//! window live together in the private, *childless* inline module [`sealed`]. The window's fields are
//! private to that module and it has no constructor, so the struct literal inside
//! [`SubscriberDeliveryMeter::close`] is the only thing in the crate that can produce one — and
//! `SubscriberDeliveryEvidence::closed` takes one. Rust makes a private field visible to its
//! declaring module **and every descendant**, so `sealed` deliberately has no children and this file
//! has no `tests` module: either would reopen exactly the door this arrangement closes.
//!
//! `ClosedWindow` goes no further than `subscriber_delivery`, because only the evidence constructor
//! ever names it — the same treatment `AxisBoundEvidence` gets in
//! [`evidence_artifact`](crate::view_read_set_campaign::evidence_artifact).

mod sealed {
    use std::sync::Arc;

    use anyhow::{Context, Result};
    use prometheus::core::Metric as _;
    use spacetimedb_sdk::unstable::CLIENT_METRICS;

    use crate::client::connected_client::ConnectedClient;
    use crate::module_artifact::bindings::SubscriptionHandle;
    use crate::view_read_set_campaign::subscriber_delivery::received_wire_delivery::ReceivedWireDelivery;
    use crate::view_read_set_campaign::subscriber_delivery::subscriber_delivery_evidence::SubscriberDeliveryEvidence;
    use crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counter::SubscriberRowCounter;
    use crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counts::SubscriberRowCounts;
    use crate::view_read_set_campaign::subscriber_delivery::wire_counter_snapshot::WireCounterSnapshot;

    /// One attempt's open measured window: the connection it meters, the tally its row callbacks
    /// feed, and what the frame histogram had accumulated when it opened.
    ///
    /// **The database label is taken, never given.** It comes from the [`ConnectedClient`]'s own
    /// retained connect input, which the SDK uses verbatim as the `db` label on its metrics. A label
    /// parameter would be a free input whose only symptom, when wrong, is an unused series reading
    /// zero at both ends — a perfectly well-formed window reporting that nothing was ever delivered.
    /// Nothing in the arithmetic could contradict it, so it is not a parameter. What makes that
    /// label unambiguous in the first place is the campaign's fresh-database, single-client
    /// execution, recorded on
    /// [`subscriber_delivery`](crate::view_read_set_campaign::subscriber_delivery).
    pub(crate) struct SubscriberDeliveryMeter {
        database_name: String,
        rows: Arc<SubscriberRowCounter>,
        opened: WireCounterSnapshot,
    }

    /// Proof that a meter was consumed at its close, carrying what that close measured.
    ///
    /// This type exists for one reason: to make "delivery evidence is minted only by consuming a
    /// live meter" a fact about the type system rather than a claim in a doc comment. A test that
    /// assembled row counts and a wire delta by hand would still have nothing to pass.
    pub(in crate::view_read_set_campaign::subscriber_delivery) struct ClosedWindow {
        rows: SubscriberRowCounts,
        wire: ReceivedWireDelivery,
    }

    impl SubscriberDeliveryMeter {
        /// Open a window on `client`'s connection, taking the histogram's coherent reading as the
        /// window's start.
        ///
        /// Opened before the cold subscription is issued, so that subscription's initial snapshot
        /// counts as delivered rows — which is the point: the snapshot *is* delivery, and a window
        /// that started after it would report a subscriber that received almost nothing.
        pub(crate) fn open(
            client: &ConnectedClient,
            rows: Arc<SubscriberRowCounter>,
        ) -> Result<Self> {
            let database_name = client.database_name().to_string();
            let opened = collect(&database_name).context(
                "reading the subscriber's frame histogram as the measured window opened",
            )?;
            Ok(Self {
                database_name,
                rows,
                opened,
            })
        }

        /// Close the window and mint its evidence, consuming the meter so a second close is not a
        /// state that exists.
        ///
        /// Takes the concrete subscription handles the attempt retained rather than a count of them;
        /// how many were still active is derived where the handles are, at the evidence constructor.
        ///
        /// Called after the saturated batch has confirmed and the client cache has been read, so the
        /// row snapshot taken here and the retained composition artifacts describe the same state.
        pub(crate) fn close(
            self,
            handles: &[SubscriptionHandle],
        ) -> Result<SubscriberDeliveryEvidence> {
            let closed = collect(&self.database_name).context(
                "reading the subscriber's frame histogram as the measured window closed",
            )?;
            let wire = closed.since(&self.opened).context(
                "reducing the subscriber's frame histogram to the traffic received over the \
                 measured window",
            )?;
            Ok(SubscriberDeliveryEvidence::closed(
                ClosedWindow {
                    rows: self.rows.snapshot(),
                    wire,
                },
                handles,
            ))
        }
    }

    impl ClosedWindow {
        /// What the attempt's row callbacks had counted when the window closed.
        pub(in crate::view_read_set_campaign::subscriber_delivery) fn rows(
            &self,
        ) -> SubscriberRowCounts {
            self.rows
        }

        /// The frames and bytes the connection received over the window.
        pub(in crate::view_read_set_campaign::subscriber_delivery) fn wire(
            &self,
        ) -> ReceivedWireDelivery {
            self.wire
        }
    }

    /// Take one coherent reading of `database_name`'s frame histogram.
    ///
    /// **Why this goes through `metric` rather than the two accessor methods.** `get_sample_count`
    /// and `get_sample_sum` are separate reads of separate atomics that the SDK's message-processing
    /// thread updates in sequence — and worse, `HistogramCore::observe` bumps the overall count
    /// *before* it adds the sum, so a reader can see a count that the sum has not caught up with. No
    /// number of re-reads can prove a writer is not paused inside that gap; it can only make it less
    /// likely, which is not a proof and is not used here.
    ///
    /// `Metric::metric` calls `HistogramCore::proto`, which takes the collect lock, flips the hot and
    /// cold shards, and then **spins until the cold shard's count equals the global count captured at
    /// the flip** — precisely the condition that every observation counted has finished adding its
    /// sum. Only then does it read the pair. So the returned count and sum describe one consistent
    /// state, by Prometheus's own documented protocol.
    ///
    /// Collecting is not a read-only operation on the registry: it flips which shard is hot and
    /// folds the cold one's totals into it. What moves is shard placement, not the numbers — the
    /// cumulative count and sum are preserved exactly — and `proto` serializes on the histogram's
    /// own `collect_lock`, so another collector in the process changes neither this reading's
    /// coherence nor its value.
    fn collect(database_name: &str) -> Result<WireCounterSnapshot> {
        let label: Box<str> = database_name.into();
        let metric = CLIENT_METRICS
            .websocket_received_msg_size
            .with_label_values(&label)
            .metric();
        let histogram = metric.get_histogram().as_ref().with_context(|| {
            format!(
                "the message-size histogram for database {database_name} collected a metric \
                 carrying no histogram payload"
            )
        })?;

        Ok(WireCounterSnapshot {
            frames: histogram.get_sample_count(),
            byte_sum: histogram.get_sample_sum(),
        })
    }
}

// The meter's first and only consumer is the driver's measurement stage, which now exists — so the
// re-export it deferred lands here with it.
//
// `ClosedWindow` is named by the evidence constructor, so it goes exactly that far and no further —
// the treatment `AxisBoundEvidence` gets for the same reason.
pub(in crate::view_read_set_campaign::subscriber_delivery) use sealed::ClosedWindow;
pub(crate) use sealed::SubscriberDeliveryMeter;
