//! A normalized correlation coefficient, proven to lie in `[-1, 1]` at its mint boundary.

use serde::Serialize;

use crate::analysis::finite_f64::FiniteF64;

/// How far a correctly-rounded evaluation of a Cauchy–Schwarz-bounded ratio may exceed `1`.
///
/// **Derived, not chosen.** With unit roundoff `u = EPSILON / 2`, the evaluation path is: three
/// `i128 -> f64` conversions — the numerator and both energy terms — then one multiply, one `sqrt`,
/// and one divide. Each is correctly rounded, so each contributes a factor `(1 + δ)` with
/// `|δ| <= u`. The numerator's conversion and the division act directly on the result, giving
/// `(1 + u)²`; the two energy conversions and their product enter through the square root, giving
/// `(1 - u)^(-3/2)`, and the `sqrt` itself adds `(1 - u)^(-1)`. Collecting them, the exact ratio `r`
/// with `|r| <= 1` is returned as `r · F` where
///
/// ```text
/// F <= (1 + u)² · (1 - u)^(-5/2) <= (1 + u)² / (1 - u)³
/// ```
///
/// the last step because `x -> (1 - u)^(-x)` increases in `x` for `0 < 1 - u < 1`. That integer
/// power admits an elementary bound with no asymptotics: `(1 + u)² <= (1 + 8u)(1 - u)³` reduces to
/// `0 <= u(3 - 22u + 23u² - 8u³)`, and for `u <= 1/8` the bracket is at least
/// `3 - 2.75 - 0.016 > 0.23 > 0`. Hence `F - 1 <= 8u = 4 · EPSILON` for every `u <= 1/8`, and
/// `u = 2^-53` here.
const RATIO_ROUNDING_CEILING: f64 = 4.0 * f64::EPSILON;

/// A coefficient in `[-1, 1]`, finite by construction.
///
/// **The range is the invariant.** [`FiniteF64`] proves finiteness but admits `2.0`, and a
/// normalized autocorrelation outside `[-1, 1]` is not a value this report can mean. Wrapping it
/// makes the out-of-range state unrepresentable rather than merely undocumented — the previous
/// documentation asserted the range while nothing enforced it, and an admissible thousand-sample
/// series at lag 999 really did serialize `-1.0000000000000002`.
///
/// `#[serde(transparent)]` over a transparent [`FiniteF64`] renders this as the bare number, so the
/// artifact's shape is unchanged and only its domain is tightened.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct CorrelationCoefficient(FiniteF64);

impl CorrelationCoefficient {
    /// Mint from the `f64` evaluation of a ratio the exact integers already prove is in `[-1, 1]`.
    ///
    /// Named for the *approximation* it receives, not for the exact ratio behind it: the caller has
    /// already rounded, and this boundary repairs only that rounding.
    ///
    /// **What gating the clamp does and does not buy.** The clamp is bounded by
    /// `RATIO_ROUNDING_CEILING`, a bound derived from the operation sequence rather than observed
    /// from data, so a deviation *larger than* that ceiling — the shape a sign error, a wrong slice
    /// offset, or a broken centring identity typically produces — fails loudly instead of being
    /// pulled to the boundary. It does **not** establish that every implementation defect exceeds
    /// the ceiling: one landing within a few ULPs of the boundary is indistinguishable from rounding
    /// and would be repaired silently. An unconditional clamp would absorb every case and is
    /// deliberately not what this does.
    ///
    /// The exact numerator and both energy terms are retained separately by the containing report
    /// and remain the authority; this value is the derived convenience projection.
    pub(crate) fn from_rounded_ratio(approximation: f64) -> Self {
        assert!(
            approximation.is_finite(),
            "a normalized coefficient must be finite; got {approximation}"
        );
        assert!(
            approximation.abs() <= 1.0 + RATIO_ROUNDING_CEILING,
            "a Cauchy-Schwarz-bounded ratio cannot exceed 1 by more than the derived rounding \
             ceiling {RATIO_ROUNDING_CEILING:e}; {approximation} indicates a defect in the \
             normalization or evaluation path, not rounding — the exact components may themselves \
             still be correct"
        );
        Self(FiniteF64::new(approximation.clamp(-1.0, 1.0)))
    }
}

#[cfg(test)]
mod tests;
