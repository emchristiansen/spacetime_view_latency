//! The obligatory linear per-run cleanup owner and its terminal-minting witness.
//!
//! A run's execution yields an inert pre-cleanup carrier; [`RunCleanup`] is the sole capability that can
//! consume one, and it always attempts the ordered client-disconnect-then-server-teardown before minting
//! the run terminal. The [`CleanupMinted`] witness — whose constructor is module-private here — is the
//! structural proof that no other code can mint a run terminal. This entry file is declarative only.

mod cleanup_minted;
mod must_disconnect;
mod run_cleanup;

pub(crate) use cleanup_minted::CleanupMinted;
pub(crate) use run_cleanup::RunCleanup;

/// No-I/O reported-cleanup settlement stand-ins for the cursor tests. Reported, not effected — see their
/// docs; gated `#[cfg(test)]` so no production path can reach them.
#[cfg(test)]
pub(in crate::campaign) use run_cleanup::{settle_executed_reported, settle_stopped_reported};
