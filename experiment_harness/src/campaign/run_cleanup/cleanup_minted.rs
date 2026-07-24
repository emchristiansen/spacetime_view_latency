//! The unforgeable witness that a run terminal is being minted by the obligatory linear cleanup owner.

/// Proof that a run terminal ([`RunComplete`](crate::campaign::run_cursor::RunComplete),
/// [`RunDone`](crate::campaign::run_cursor::RunDone),
/// [`RunIncomplete`](crate::campaign::run_cursor::RunIncomplete)) is being minted from *inside* the
/// `run_cleanup` module — i.e. by [`RunCleanup`](super::RunCleanup)'s settlement, after it has attempted
/// the ordered client-disconnect-then-server-teardown.
///
/// Each terminal constructor takes one by value. The token is a private-field zero-sized type whose only
/// constructor, [`Self::new`], is `pub(super)` — visible solely within the `run_cleanup` module. So no
/// code outside `run_cleanup` can produce a `CleanupMinted`, and therefore no code outside `run_cleanup`
/// can mint a run terminal, no matter how visible the terminal constructors themselves are. This is the
/// structural proof of the invariant "`RunCleanup` is the sole path that can mint `RunDone`/`RunIncomplete`":
/// the run cursor's own transitions can build the inert pre-cleanup carriers
/// ([`RunExecuted`](crate::campaign::run_cursor::RunExecuted) /
/// [`RunExecutionStopped`](crate::campaign::run_cursor::RunExecutionStopped)) but cannot forge this witness
/// to turn one into a terminal.
pub(crate) struct CleanupMinted(());

impl CleanupMinted {
    /// Mint the witness. `pub(super)` so only the `run_cleanup` module — the linear cleanup owner and its
    /// explicitly `#[cfg(test)]` reported-cleanup stand-ins — can produce one.
    pub(super) fn new() -> Self {
        Self(())
    }
}
