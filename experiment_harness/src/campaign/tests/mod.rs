//! Tests for the consuming-typestate campaign → block → run cursors, driven through the real sink
//! methods with a no-I/O scripted writer. One test entity per file.
//!
//! These prove the *typestate structure*: exact run/manifest/dose coordinate binding, ten-dose ladder
//! and two-run block exhaustion, whole 270-block / 540-run schedule exhaustion and order, foreign-
//! coordinate/dose rejection, the always-attempted finalize gating completion, execution + finalization
//! dual-failure preservation, and finalization-only failure retaining typed progress. They do not — and
//! a unit test cannot — prove crash durability; the scripted writer only *returns* success or failure.
//!
//! Physical-table cardinalities and the raw latency/summary/event payload the fixtures carry are
//! internally-consistent evidence derived through the real dataset/dose APIs; they are never a success
//! condition for any cursor transition.

mod drive;
mod driving_writer;

mod dose_write_failure_yields_dosing_frontier;
mod finalization_only_failure_retains_typed_progress;
mod full_schedule_drives_to_campaign_complete;
mod manifest_write_failure_yields_execution_frontier;
mod write_manifest_rejects_a_foreign_coordinate;
mod write_observation_rejects_a_foreign_dose;
