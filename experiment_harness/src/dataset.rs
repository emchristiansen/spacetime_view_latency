//! Deterministic multi-identity dataset: seed plan, expected result sets, and read-back.
//!
//! The seed plan is decided from typed role identities and the run's cell, independently of
//! the live client. The same [`subscribed_table::SubscribedTable`] decision drives the
//! subscription SQL, the seed-derived expected set, and the cache read-back, so a run's
//! correctness check cannot silently disagree with what it subscribed to.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub mod campaign_dataset;
pub mod chronicle_physical_rows;
pub mod dose_batch;
pub mod dose_index;
pub mod message_physical_rows;
pub mod physical_cardinalities;
pub mod role_slice;
pub mod run_dataset;
pub mod seed_op;
pub mod seed_plan;
pub mod seeded_visibility;
pub mod subscribed_rows;
pub mod subscribed_table;
