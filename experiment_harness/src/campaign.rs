//! The consuming-typestate campaign driver: private campaign → block → run cursor ownership.
//!
//! The schedule owns 270 repetition blocks (9 cells × 30 blocks); each block owns its exact adjacent
//! `[Run; 2]`; each run owns the fixed ten-dose ladder plus its manifest/output state. This module
//! turns that ownership into a driver whose *invalid transitions are unrepresentable*:
//!
//! - Every cursor level is a set of **consuming typestates**. A transition takes `self` by value and
//!   yields the next state (or a terminal), so there is no `&mut self` + `Option<active>` guard to
//!   misuse — calling a transition out of order does not type-check.
//! - Progress is gated on **iterator exhaustion**, never a count: a campaign completes iff its
//!   schedule iterator is drained *and* no block is outstanding; a block iff its `[Run; 2]` iterator
//!   is drained *and* no run is outstanding; a run iff its dose iterator is drained.
//! - Completion receipts ([`run_cursor::RunComplete`], [`block_cursor::BlockComplete`],
//!   [`campaign_cursor::CampaignComplete`]) are affine (no `Clone`) with `pub(super)` constructors, so
//!   only the exhausting transition mints one — arbitrary counts or collections cannot fabricate
//!   success, and a completion cannot be replayed.
//! - The single required output stream ([`ObservationSink`](crate::observation::observation_sink)) is
//!   owned by the campaign and threaded by move through the run states, so its write receipts never
//!   escape to a sibling and it is always recovered for exactly one finalize.
//! - A run advances `WritingManifest → Dosing → (exhausted | stopped)`, then its linear cleanup owner
//!   ([`run_cleanup::RunCleanup`]) attempts the ordered client disconnect then server teardown and mints
//!   the terminal; `RunComplete` is minted only after a clean cleanup, so a run cannot complete before its
//!   client and server have been cleaned up, and a simultaneous execution-and-cleanup failure is retained
//!   as typed structure.
//!
//! This entry file is declarative module declarations only.

mod block_frontier;
mod campaign_aborted;
mod campaign_frontier;
mod campaign_incomplete;
mod campaign_outcome;
mod campaign_partial_path;
mod campaign_root;
mod effect_stage;
mod finalization_outcome;
mod latest_progress;
mod run_acquisition_failure;
mod run_cleanup_failure;
mod run_cleanup_outcome;
mod run_frontier;
mod run_incompletion;
mod run_stage;
mod run_stop_evidence;
mod sink_write_stage;

mod block_cursor;
mod campaign_cursor;
mod run_campaign;
mod run_cleanup;
mod run_cursor;
mod run_driver;

pub(crate) use campaign_outcome::CampaignOutcome;
pub(crate) use campaign_partial_path::CampaignPartialPath;
pub(crate) use campaign_root::CampaignRoot;
pub(crate) use run_campaign::run_campaign;

#[cfg(test)]
mod tests;
