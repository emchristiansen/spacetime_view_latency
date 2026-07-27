//! Supporting evidence about the measured subscriber's connection over one attempt's window.
//!
//! The spec asks for rows and bytes delivered, client-cache rows, and subscription handles as
//! *supporting* evidence rather than trend estimands. Client-cache rows are already derived from the
//! composition census, which is replayable; everything here is not. These are live facts about one
//! connection over one window, so they are recorded beside the composition finding rather than
//! inside it — the separation checkpoint `a598e310` made structural.
//!
//! **The pipeline, and where each fact comes from.**
//!
//! - Delivered rows come from [`subscriber_row_counter::SubscriberRowCounter`], a campaign-local
//!   tally the SDK's own row callbacks increment. The three kinds stay separate all the way to the
//!   ledger; only their sum is derived.
//! - Bytes and frames come from one coherent reading of the pinned SDK's
//!   `websocket_received_msg_size` histogram, taken at each end of the window into a
//!   [`wire_counter_snapshot::WireCounterSnapshot`] and reduced to a
//!   [`received_wire_delivery::ReceivedWireDelivery`] by arithmetic that refuses every reading it
//!   cannot convert exactly.
//! - Active subscription handles are counted from the concrete handles themselves. No count is
//!   accepted anywhere in this module.
//!
//! **"Bytes" means compressed whole-connection frame bytes.** The SDK records a frame's length
//! before `parse_response` decompresses it, and counts ping, pong, and unexpected frames alongside
//! binary ones, so this is on-the-wire traffic for the whole connection over the window — not row
//! payload bytes, and not a per-row figure. Encoding rows in callbacks to get payload bytes would
//! add serialization to the exact path E2 measures, so it is not done: the histogram costs nothing
//! extra because the SDK already pays it unconditionally. That histogram's sample count *is* the
//! frame count, so the SDK's separate `websocket_received` counter is redundant and is never read;
//! why it could not have been read coherently anyway — and why `prometheus` is therefore a direct
//! dependency — is at [`subscriber_delivery_meter`], where the reading happens.
//!
//! **The registry is process-global, which is a precondition, not an implementation detail.**
//! `CLIENT_METRICS` is a `Lazy` static shared by every connection in the process, labelled only by
//! database name. Two clients on one database, or a database name reused across attempts, would sum
//! into the same series. The campaign satisfies this by construction — every scale point publishes a
//! fresh database and one attempt connects one measured subscriber at a time — and that sequential
//! fresh-database, single-client execution is what makes a window's delta attributable to the
//! attempt that opened it.
//!
//! **What the seal costs.** Evidence cannot be minted without consuming a live meter; the mechanism
//! is at [`subscriber_delivery_meter`] and [`subscriber_delivery_evidence`], where it is enforced.
//! The price is paid here: the evidence type and its derived total are compile-checked and reviewed
//! by inspection rather than exercised at runtime, while the arithmetic and the row counter — where
//! a mistake would actually hide — stay purely and exhaustively tested. The two carrier values in
//! between are named-field bundles rather than positional constructors, since both hold same-typed
//! counts whose silent swap would misreport the ledger.
//!
//! **What does not live here yet.** Registering the row callbacks, retaining subscription handles,
//! and opening and closing the meter around the four channels are the driver's measurement stage,
//! which is still a `todo!()`. This module is the vocabulary that stage will use.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod received_wire_delivery;
pub(crate) mod subscriber_delivery_evidence;
pub(crate) mod subscriber_delivery_meter;
pub(crate) mod subscriber_row_counter;
pub(crate) mod subscriber_row_counts;
pub(crate) mod wire_counter_snapshot;
