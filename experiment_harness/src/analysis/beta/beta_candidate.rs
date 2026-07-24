//! A fitted `(β, a, b, RSS)` quadruple at one exponent.

/// The constrained least-squares fit at one fixed exponent `β`: the nonnegative intercept `a`, the
/// strictly positive scale `b`, and the residual sum of squares `RSS` of `y ≈ a + bN^β` over a block's
/// ten-dose ladder. A `BetaCandidate` exists only for a `β` whose constrained fit is finite with `b > 0`
/// (spec: "no fit exists when no candidate has strictly positive finite `b`").
///
/// The fields are sealed with no crate-wide minter: [`Self::new`] is `pub(super)`, so a candidate is
/// assembled only from within the `beta` module — in practice only by
/// [`constrained_fit`](super::constrained_fit::constrained_fit) — and it asserts the full invariant
/// (finite `β`; finite `a ≥ 0`; finite `b > 0`; finite `RSS ≥ 0`). An invalid candidate is therefore
/// unrepresentable rather than prevented by caller discipline.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BetaCandidate {
    /// The exponent this fit was solved at (finite).
    beta: f64,
    /// The fitted nonnegative intercept `a ≥ 0` (finite).
    a: f64,
    /// The fitted strictly positive scale `b > 0` (finite).
    b: f64,
    /// The residual sum of squares of `y − (a + bN^β)` over the ladder (finite, `≥ 0`).
    rss: f64,
}

impl BetaCandidate {
    /// Bind a fitted quadruple, asserting its invariant at the mint boundary. `pub(super)` so only the
    /// `beta` module's constrained-fit solver can produce one; the asserts make a NaN/∞ component, a
    /// negative intercept, a non-positive scale, or a negative residual sum a loud failure rather than a
    /// representable candidate.
    pub(super) fn new(beta: f64, a: f64, b: f64, rss: f64) -> Self {
        assert!(beta.is_finite(), "a fitted exponent β must be finite, got {beta}");
        assert!(
            a.is_finite() && a >= 0.0,
            "a fitted intercept a must be finite and nonnegative, got {a}"
        );
        assert!(
            b.is_finite() && b > 0.0,
            "a fitted scale b must be finite and strictly positive, got {b}"
        );
        assert!(
            rss.is_finite() && rss >= 0.0,
            "a fitted residual sum of squares must be finite and nonnegative, got {rss}"
        );
        Self { beta, a, b, rss }
    }

    /// The exponent this fit was solved at.
    pub(crate) fn beta(self) -> f64 {
        self.beta
    }

    /// The fitted nonnegative intercept `a`.
    pub(crate) fn a(self) -> f64 {
        self.a
    }

    /// The fitted strictly positive scale `b`.
    pub(crate) fn b(self) -> f64 {
        self.b
    }

    /// The residual sum of squares at this fit.
    pub(crate) fn rss(self) -> f64 {
        self.rss
    }
}
