//! The exact absolute value of a rational.

use crate::analysis::stats::rational::Rational;

/// `|value|`, exactly.
///
/// [`Rational`] deliberately offers no `abs` — the campaign's estimator never needs one, because its
/// magnitudes are latencies and its comparisons are signed. Deviations do need it: "how far is this
/// window median from the centre" is a distance, and taking it as a signed difference would let a
/// window below the centre and one above cancel in a maximum.
///
/// Implemented through the type's own `neg`, so the canonical form and its loud overflow behaviour
/// are preserved rather than reconstructed by hand from the components.
pub(crate) fn absolute_value(value: Rational) -> Rational {
    match value < Rational::zero() {
        true => value.neg(),
        false => value,
    }
}
