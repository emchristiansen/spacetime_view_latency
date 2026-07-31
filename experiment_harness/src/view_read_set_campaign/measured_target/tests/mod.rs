//! Focused tests for the role→target coupling. One test entity per file.
//!
//! Provable without a server: which target a role selects, and that each target's query names its
//! own table. The read-back, the delivery-callback registration, and the observer all take a live
//! `DbConnection` and are covered by inspection of the total matches in
//! [`MeasuredTarget`](super::MeasuredTarget) — the ceiling
//! [`SubscribedTable`](crate::dataset::subscribed_table)'s tests draw around `register_events`.

mod each_role_measures_its_own_target;
mod each_target_subscribes_to_the_table_it_names;
