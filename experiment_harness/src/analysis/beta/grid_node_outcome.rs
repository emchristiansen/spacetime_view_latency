//! One coarse-grid node's constrained-fit outcome, with no non-finite sentinel.

use crate::analysis::finite_f64::FiniteF64;

/// The constrained-fit outcome at a single frozen grid node. The infeasible case is a *typed* variant,
/// never a `+∞`/NaN `f64` sentinel: non-finite floats are not valid authoritative JSON, and a sentinel
/// would let an invalid machine-readable state (an "RSS" that is not a residual sum) be serialized. So a
/// node either carries a finite, nonnegative residual sum or is explicitly
/// [`NoPositiveScale`](Self::NoPositiveScale).
///
/// This mirrors the search's own `eval_rss` semantics — a node with no constrained fit is treated as
/// worse than any real one — without materializing the `+∞` it used internally. Analysis-domain
/// telemetry, not a report DTO: not `Serialize`; the report projects it across the lossy boundary.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GridNodeOutcome {
    /// A constrained fit with strictly positive finite scale exists at this node; its residual sum of
    /// squares is a finite, nonnegative [`FiniteF64`]. Mint only via [`Self::feasible`], which asserts
    /// nonnegativity.
    Feasible {
        /// The finite, nonnegative residual sum of squares of the node's constrained fit.
        rss: FiniteF64,
    },
    /// No constrained fit with strictly positive finite scale exists at this node.
    NoPositiveScale,
}

impl GridNodeOutcome {
    /// A feasible node, asserting the residual sum is nonnegative at the mint boundary. `pub(super)` so
    /// only the `beta` module's search constructs one — an RSS is by definition `Σ residual² ≥ 0`, so a
    /// negative value is a loud panic rather than a representable telemetry state.
    pub(super) fn feasible(rss: FiniteF64) -> Self {
        assert!(
            rss.get() >= 0.0,
            "a grid node's residual sum of squares must be nonnegative, got {}",
            rss.get()
        );
        Self::Feasible { rss }
    }

    /// Whether this node admitted a constrained fit — the property a located basin node must satisfy.
    pub(crate) fn is_feasible(self) -> bool {
        matches!(self, Self::Feasible { .. })
    }

    /// This node's finite, nonnegative residual sum of squares when it admitted a constrained fit, or
    /// `None` when no constrained fit with strictly positive finite scale exists. A narrow read accessor
    /// so the report projects a feasible node's already-[`FiniteF64`] RSS without matching the private
    /// variant, keeping the infeasible case a typed `None` rather than a sentinel.
    pub(crate) fn feasible_rss(self) -> Option<FiniteF64> {
        match self {
            Self::Feasible { rss } => Some(rss),
            Self::NoPositiveScale => None,
        }
    }
}
