//! Why one refined local basin's golden-section refinement stopped.

/// The termination cause of the golden-section refinement of one grid-local RSS basin (spec: retain, per
/// refined basin, `BracketWidthReached` versus `IterationCapReached`). The two variants mirror the two
/// clauses of the frozen `while (hi - lo) > GOLDEN_BRACKET_TOL && iters < GOLDEN_MAX_ITERS` loop guard in
/// [`fit_block`](super::fit_block), so the recorded cause can never be a state the search cannot reach.
///
/// This is analysis-domain telemetry, not a report DTO: it is not `Serialize`; the report projects it
/// into its own beta convergence report types across the lossy boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GoldenStop {
    /// The bracket contracted to at most the frozen `GOLDEN_BRACKET_TOL` before the iteration cap — the
    /// refinement converged to tolerance. This is an optimizer-convergence cause only; it is unrelated to
    /// the separate post-search [`PinnedAtBound`](super::block_fit::BlockFit::PinnedAtBound)
    /// identifiability outcome.
    BracketWidthReached,
    /// The frozen `GOLDEN_MAX_ITERS` cap was hit while the bracket was still wider than the tolerance —
    /// the refinement was truncated, not converged.
    IterationCapReached,
}
