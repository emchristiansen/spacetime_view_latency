//! The consuming-typestate campaign cursor: run the whole seeded schedule's blocks to completion.
//!
//! The campaign owns the complete [`Schedule`](crate::plan::schedule::Schedule) block order *and* the
//! single [`ObservationSink`](crate::observation::observation_sink) by move, threading the sink into
//! each block in turn and recovering it when the block hands back a
//! [`BlockDone`](crate::campaign::block_cursor::BlockDone) or
//! [`BlockIncomplete`](crate::campaign::block_cursor::BlockIncomplete). Progress is gated on the block
//! iterator's exhaustion, never a count: the campaign is executed-complete iff its block iterator is
//! drained *and* no block is outstanding — and even then success is gated on an explicit finalize.
//!
//! Terminal outcomes are minted only by the exhausting/aborting transitions:
//! [`CampaignExhausted::finalize`] mints affine [`CampaignComplete`] *only* when the sink finalizes
//! cleanly; any execution failure or finalize failure yields a
//! [`CampaignIncomplete`](crate::campaign::campaign_incomplete::CampaignIncomplete) that retains both
//! the execution frontier and the finalization outcome, or the typed finalize error, so no failure is
//! discarded. This entry file is declarative module declarations and re-exports only.

mod campaign_block_outcome;
mod campaign_block_pending;
mod campaign_complete;
mod campaign_exhausted;
mod campaign_ready;
mod campaign_step;

pub(crate) use campaign_block_outcome::CampaignBlockOutcome;
pub(crate) use campaign_block_pending::CampaignBlockPending;
pub(crate) use campaign_complete::CampaignComplete;
pub(crate) use campaign_exhausted::CampaignExhausted;
pub(crate) use campaign_ready::CampaignReady;
pub(crate) use campaign_step::CampaignStep;
