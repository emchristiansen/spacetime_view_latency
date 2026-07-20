//! The report projection of one fitted `(β, a, b, RSS)` candidate.

use serde::Serialize;

use crate::analysis::beta::beta_candidate::BetaCandidate;
use crate::analysis::finite_f64::FiniteF64;

/// The lossy report projection of one exact [`BetaCandidate`]: its exponent, nonnegative intercept,
/// strictly positive scale, and residual sum of squares. The primary [`BetaCandidate`] is non-`Serialize`
/// and crosses the floating-point boundary only here; every stored float is a finite-by-construction
/// [`FiniteF64`], so no NaN/±∞ can be serialized (the source candidate already proved every component
/// finite at its mint boundary).
#[derive(Debug, Serialize)]
pub(crate) struct BetaCandidateReport {
    /// The exponent this fit was solved at.
    beta: FiniteF64,
    /// The fitted nonnegative intercept `a`.
    a: FiniteF64,
    /// The fitted strictly positive scale `b`.
    b: FiniteF64,
    /// The residual sum of squares at this fit.
    rss: FiniteF64,
}

impl BetaCandidateReport {
    /// Project one exact candidate into its finite report shape. Takes the `Copy` [`BetaCandidate`] by
    /// value — one input.
    pub(crate) fn of(candidate: BetaCandidate) -> Self {
        // The source candidate already proved every component finite at its mint boundary, so each
        // `FiniteF64::new` assertion can never fire here.
        Self {
            beta: FiniteF64::new(candidate.beta()),
            a: FiniteF64::new(candidate.a()),
            b: FiniteF64::new(candidate.b()),
            rss: FiniteF64::new(candidate.rss()),
        }
    }
}
