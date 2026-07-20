//! The complete refinement telemetry of one grid-local RSS basin.

use crate::analysis::beta::fit_block::{GOLDEN_BRACKET_TOL, GOLDEN_MAX_ITERS};
use crate::analysis::beta::golden_stop::GoldenStop;
use crate::analysis::finite_f64::FiniteF64;

/// The termination telemetry of the golden-section refinement of a single grid-local RSS basin (spec:
/// "termination telemetry for every refined local basin (initial/final brackets, iteration count, and
/// `BracketWidthReached` versus `IterationCapReached`)"). Every basin the frozen search refines
/// contributes exactly one of these, so the convergence record covers the *complete* global search, not
/// only the winning basin.
///
/// Fields are private with no defaults; [`Self::new`] is `pub(super)`, so a `BasinTermination` is
/// assembled only from within the `beta` module — in production only by the
/// [`fit_block`](super::fit_block) search that owns the frozen constants it records. Every bracket
/// endpoint is a finite [`FiniteF64`] (the brackets live within the `[0.1, 4.0]` domain by construction).
/// Analysis-domain telemetry, not a report DTO: not `Serialize`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BasinTermination {
    /// The 0-based index of the grid node whose local basin this refinement started from.
    grid_node_index: usize,
    /// The lower endpoint of the refinement's initial bracket (the basin node's left neighbour).
    initial_bracket_lo: FiniteF64,
    /// The upper endpoint of the refinement's initial bracket (the basin node's right neighbour).
    initial_bracket_hi: FiniteF64,
    /// The lower endpoint of the bracket at termination.
    final_bracket_lo: FiniteF64,
    /// The upper endpoint of the bracket at termination.
    final_bracket_hi: FiniteF64,
    /// The number of golden-section iterations executed before termination.
    iterations: usize,
    /// Why the refinement stopped: bracket-width tolerance versus the iteration cap.
    stop: GoldenStop,
}

impl BasinTermination {
    /// Bind one basin's refinement telemetry, asserting it is consistent with the frozen
    /// [`golden_section`](super::fit_block) control flow:
    ///
    /// - Each bracket is ordered (`lo ≤ hi`) — grid nodes ascend, and the loop only ever advances `lo`
    ///   or retreats `hi`.
    /// - The final bracket is contained in the initial one — every loop step keeps `[lo, hi]` inside
    ///   `[lo0, hi0]`.
    /// - `iterations ≤ GOLDEN_MAX_ITERS` — the loop guard is `iters < GOLDEN_MAX_ITERS`.
    /// - The stop cause matches the loop's two exit clauses `while (hi - lo) > GOLDEN_BRACKET_TOL &&
    ///   iters < GOLDEN_MAX_ITERS`: a [`BracketWidthReached`](GoldenStop::BracketWidthReached) exit
    ///   implies the final width is within `GOLDEN_BRACKET_TOL`; an
    ///   [`IterationCapReached`](GoldenStop::IterationCapReached) exit implies `iterations` reached the
    ///   cap. Implications, not biconditionals, because the two clauses can coincide on the last step.
    ///
    /// `pub(super)` so only the `beta` module's search can mint one; the asserts make an
    /// unreachable-in-the-search telemetry state a loud panic rather than representable evidence.
    pub(super) fn new(
        grid_node_index: usize,
        initial_bracket_lo: FiniteF64,
        initial_bracket_hi: FiniteF64,
        final_bracket_lo: FiniteF64,
        final_bracket_hi: FiniteF64,
        iterations: usize,
        stop: GoldenStop,
    ) -> Self {
        assert!(
            initial_bracket_lo.get() <= initial_bracket_hi.get(),
            "initial bracket is unordered: {} > {}",
            initial_bracket_lo.get(),
            initial_bracket_hi.get()
        );
        assert!(
            final_bracket_lo.get() <= final_bracket_hi.get(),
            "final bracket is unordered: {} > {}",
            final_bracket_lo.get(),
            final_bracket_hi.get()
        );
        assert!(
            initial_bracket_lo.get() <= final_bracket_lo.get()
                && final_bracket_hi.get() <= initial_bracket_hi.get(),
            "final bracket [{}, {}] is not contained in the initial bracket [{}, {}]",
            final_bracket_lo.get(),
            final_bracket_hi.get(),
            initial_bracket_lo.get(),
            initial_bracket_hi.get()
        );
        assert!(
            iterations <= GOLDEN_MAX_ITERS,
            "golden-section iterations {iterations} exceed the frozen cap {GOLDEN_MAX_ITERS}"
        );
        match stop {
            GoldenStop::BracketWidthReached => assert!(
                final_bracket_hi.get() - final_bracket_lo.get() <= GOLDEN_BRACKET_TOL,
                "a BracketWidthReached stop must leave a final width within {GOLDEN_BRACKET_TOL}, got {}",
                final_bracket_hi.get() - final_bracket_lo.get()
            ),
            GoldenStop::IterationCapReached => assert!(
                iterations == GOLDEN_MAX_ITERS,
                "an IterationCapReached stop must have executed the full {GOLDEN_MAX_ITERS} iterations, \
                 got {iterations}"
            ),
        }
        Self {
            grid_node_index,
            initial_bracket_lo,
            initial_bracket_hi,
            final_bracket_lo,
            final_bracket_hi,
            iterations,
            stop,
        }
    }

    /// The 0-based grid node this basin refined from.
    pub(crate) fn grid_node_index(&self) -> usize {
        self.grid_node_index
    }

    /// The lower endpoint of the refinement's initial bracket — the interval the returned candidate's
    /// exponent is proven to lie within (every point `golden_section` evaluates stays inside `[lo0, hi0]`).
    pub(crate) fn initial_bracket_lo(&self) -> f64 {
        self.initial_bracket_lo.get()
    }

    /// The upper endpoint of the refinement's initial bracket.
    pub(crate) fn initial_bracket_hi(&self) -> f64 {
        self.initial_bracket_hi.get()
    }

    /// The lower endpoint of the bracket at termination — contained in the initial bracket (asserted at
    /// the mint boundary).
    pub(crate) fn final_bracket_lo(&self) -> f64 {
        self.final_bracket_lo.get()
    }

    /// The upper endpoint of the bracket at termination.
    pub(crate) fn final_bracket_hi(&self) -> f64 {
        self.final_bracket_hi.get()
    }

    /// The number of golden-section iterations the refinement executed (one per completed loop body), at
    /// most the frozen [`GOLDEN_MAX_ITERS`] cap (asserted at the mint boundary).
    pub(crate) fn iterations(&self) -> usize {
        self.iterations
    }

    /// Why the refinement stopped.
    pub(crate) fn stop(&self) -> GoldenStop {
        self.stop
    }
}
