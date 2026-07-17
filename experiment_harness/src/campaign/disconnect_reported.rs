//! Evidence that the measured client's disconnect transition was reported by the run driver.

/// An unforgeable token proving a run advanced past its dose ladder to the disconnect transition.
///
/// A run cursor consumes one to move from [`RunDisconnecting`](super::run_cursor::RunDisconnecting) to
/// [`RunTeardown`](super::run_cursor::RunTeardown), so the disconnect step cannot be skipped: teardown
/// is unreachable without first surrendering this evidence.
///
/// **Reported, not effected.** Holding this token proves only that the future measurement driver
/// *reported* reaching the disconnect transition — it is not proof the measured subscriber's socket
/// actually closed. The effectful disconnect belongs to the future driver; the production constructor
/// that mints this token from that effect is future work, so this Phase-1 skeleton exposes only a
/// `#[cfg(test)]` minting path. No production caller can fabricate one, so no production path can yet
/// advance a run past the dose ladder.
#[derive(Debug)]
pub(crate) struct DisconnectReported(());

impl DisconnectReported {
    /// Mint disconnect evidence for a driven run. Test-only: the production adapter that reports a real
    /// disconnect belongs to the future measurement driver, so this milestone has no crate-callable
    /// production constructor.
    #[cfg(test)]
    pub(crate) fn reported() -> Self {
        Self(())
    }
}
