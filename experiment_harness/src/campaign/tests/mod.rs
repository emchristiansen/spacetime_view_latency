//! Tests for the run cursor's write path and its pre-cleanup carriers, driven through the real sink
//! methods with a no-I/O scripted writer. One test entity per file.
//!
//! Scope after the single-carrier redesign: the block and campaign carriers own their run/block folding
//! *privately*, reaching it only through the real effect path, so the whole-schedule and campaign-terminal
//! drives are no longer no-I/O unit concerns. What survives as a unit concern is the run cursor itself and
//! the ownership the redesign made structural:
//!
//! - `manifest`/`context` determine the cursor and its failure frontier: a manifest- or dose-write failure
//!   stops the run at the stage/coordinate the bound context's own manifest names
//!   ([`manifest_write_failure_yields_execution_frontier`], [`dose_write_failure_yields_dosing_frontier`]);
//! - [`RunDataset`](crate::dataset::run_dataset::RunDataset) co-derives its dataset and subscription target
//!   from the owned manifest, never an independent run
//!   ([`run_dataset_co_derives_dataset_from_owned_manifest`]);
//! - an awaiting-dose state assembles its observation from its own owned context and drawn active dose, so a
//!   foreign observation is unrepresentable rather than merely rejected
//!   ([`awaiting_dose_derives_and_binds_its_observation`]).
//!
//! The seed-order/dedup/determinism/cardinality (9 cells × 30 = 270 blocks) of the schedule are covered as
//! pure schedule combinators in `plan::schedule::tests`; the cleanup-settlement invariants are covered in
//! `campaign::run_cleanup`'s own test tree, which drives real pre-cleanup carriers through the shared
//! [`drive`] helpers. These tests do not — and a unit test cannot — prove crash durability; the scripted
//! writer only *returns* success or failure.
//!
//! Physical-table cardinalities and the raw latency/summary/event payload the fixtures carry are
//! internally-consistent evidence derived through the real dataset/dose APIs; they are never a success
//! condition for any cursor transition.

mod capturing_line_writer;
pub(in crate::campaign) mod drive;
mod driving_writer;

mod awaiting_dose_derives_and_binds_its_observation;
mod dose_write_failure_yields_dosing_frontier;
mod manifest_write_failure_yields_execution_frontier;
mod run_dataset_co_derives_dataset_from_owned_manifest;
