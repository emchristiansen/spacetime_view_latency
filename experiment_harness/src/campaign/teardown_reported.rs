//! Evidence that the isolated server's teardown transition was reported by the run driver.

/// An unforgeable token proving a run advanced past its disconnect to the server-teardown transition.
///
/// A run cursor consumes one to move from [`RunTeardown`](super::run_cursor::RunTeardown) to its
/// completion, so [`RunComplete`](super::run_cursor::RunComplete) can be minted only after teardown
/// evidence is surrendered: a run cannot complete before disconnect *and* teardown.
///
/// **Reported, not effected.** Holding this token proves only that the future measurement driver
/// *reported* reaching the teardown transition — it is not proof the isolated server process actually
/// exited or its data directory was removed. The effectful teardown belongs to the future driver; the
/// production constructor that mints this token from that effect is future work, so this Phase-1
/// skeleton exposes only a `#[cfg(test)]` minting path. No production caller can fabricate one, so no
/// production path can yet complete a run.
#[derive(Debug)]
pub(crate) struct TeardownReported(());

impl TeardownReported {
    /// Mint teardown evidence for a driven run. Test-only: the production adapter that reports a real
    /// teardown belongs to the future measurement driver, so this milestone has no crate-callable
    /// production constructor.
    #[cfg(test)]
    pub(crate) fn reported() -> Self {
        Self(())
    }
}
