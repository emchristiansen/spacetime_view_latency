//! The consuming-typestate block cursor: run this block's exact `[Run; 2]`, in order, to completion.
//!
//! A block owns the campaign's [`ObservationSink`](crate::observation::observation_sink) by move and
//! threads it into each run in turn, recovering it when the run hands back a
//! [`RunDone`](crate::campaign::run_cursor::RunDone) or
//! [`RunIncomplete`](crate::campaign::run_cursor::RunIncomplete). Progress is gated on the `[Run; 2]`
//! iterator's exhaustion, never a count: the block completes iff its two runs are both drawn *and* the
//! last one completed. [`BlockComplete`] is affine and minted only at that exhaustion edge. This entry
//! file is declarative module declarations and re-exports only.

mod block_complete;
mod block_done;
mod block_incomplete;
mod block_ready;
mod block_running_run;
mod block_step;

pub(crate) use block_complete::BlockComplete;
pub(crate) use block_done::BlockDone;
pub(crate) use block_incomplete::BlockIncomplete;
pub(crate) use block_ready::BlockReady;
pub(crate) use block_running_run::BlockRunningRun;
pub(crate) use block_step::BlockStep;
