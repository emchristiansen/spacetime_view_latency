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
//! - A run advances `WritingManifest → Dosing → Disconnecting → Teardown → complete`; `RunComplete` is
//!   minted only by the teardown transition, so a run cannot complete before disconnect and teardown.
//!
//! This entry file is declarative module declarations only.

mod block_frontier;
mod campaign_frontier;
mod campaign_incomplete;
mod campaign_outcome;
mod disconnect_reported;
mod finalization_outcome;
mod latest_progress;
mod run_frontier;
mod run_stage;
mod teardown_reported;

mod block_cursor;
mod campaign_cursor;
mod run_cursor;

#[cfg(test)]
mod tests;
