//! How a run's obligatory post-execution cleanup failed: an ordered disconnect-then-teardown result.

use anyhow::Error;

use super::run_stage::RunStage;

/// Which part of the ordered per-run cleanup failed. Cleanup is always attempted in order —
/// disconnect the measured client first, then tear down the isolated server + staged WASM — and
/// **both steps always run** even if the first fails, so a disconnect failure can never hide a
/// teardown failure.
///
/// The variants make the ordering structural and rule out an invalid "both absent" state (which a
/// `Failed { disconnect: Option<E>, teardown: Option<E> }` shape would admit):
///
/// - [`Self::Disconnect`]: the client disconnect failed. The teardown was still attempted, so its
///   result is retained verbatim — the `Ok(())` of a teardown that nonetheless succeeded, or the
///   `Err` of one that also failed (the dual-cleanup-failure case), never dropped.
/// - [`Self::Teardown`]: the client disconnected cleanly but the server/staged composite teardown
///   failed.
///
/// Holds `anyhow::Error` and so is `Debug`-only (no `Clone`/`Eq`) — matching its campaign-level
/// siblings [`CampaignIncomplete`](super::campaign_incomplete::CampaignIncomplete) and
/// [`FinalizationOutcome`](super::finalization_outcome::FinalizationOutcome).
#[derive(Debug)]
pub(crate) enum RunCleanupFailure {
    /// The client disconnect failed; the still-attempted teardown's result is retained.
    Disconnect {
        error: Error,
        teardown: Result<(), Error>,
    },
    /// The client disconnected cleanly but the server/staged teardown failed.
    Teardown(Error),
}

impl RunCleanupFailure {
    /// The lifecycle stage this cleanup failure occurred in. Borrows and reads only the variant
    /// tag, returning the `Copy` [`RunStage`], so a settle transition can read the stage into a
    /// frontier and *then* move the `Error`-bearing failure exactly once — no `Error: Clone` needed.
    pub(in crate::campaign) fn stage(&self) -> RunStage {
        match self {
            RunCleanupFailure::Disconnect { .. } => RunStage::Disconnecting,
            RunCleanupFailure::Teardown(_) => RunStage::Teardown,
        }
    }
}
