//! The report projection of an exact rational, as its canonical components.

use serde::Serialize;

use crate::analysis::stats::rational::Rational;

/// An exact [`Rational`] rendered losslessly as its canonical numerator and strictly positive
/// denominator.
///
/// **Deliberately not a float.** The campaign's report projections render exact rationals through
/// [`FiniteF64`](crate::analysis::finite_f64::FiniteF64) because they are *display* values whose
/// classification input stays in the analysis domain. Here the number is the evidence: Control reads
/// these window medians and deviations to freeze `W`, and a lossy rendering would make two
/// distinguishable series look identical at the boundary where the decision is actually made.
/// `Rational` is canonical — positive denominator, lowest terms — so this projection is unique for a
/// given value and two reports are comparable field by field.
///
/// **The exact domain is exactly what checked `i128` arithmetic can represent, and nothing
/// narrower.** No ceiling is
/// claimed here, and none is derived from the measurement magnitudes: admission fixes the sample
/// count, positivity, and population, never a sample's size. Every arithmetic step that produced
/// these components was a `checked_*` operation that fails loud, so a series that leaves the domain
/// crashes rather than wrapping. Concretely, every candidate width is even, so each window median
/// evaluates `sorted[n/2 - 1].add(sorted[n/2]).div_int(2)`, and
/// [`Rational::add`](crate::analysis::stats::rational::Rational::add) panics on the checked sum — a
/// series whose two central order statistics sum beyond `i128` is outside what that checked
/// arithmetic can *compute*, rather than outside what these components could hold, and it says so
/// loudly instead of wrapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct ExactRationalReport {
    /// The canonical numerator; the sign lives here.
    numerator: i128,
    /// The canonical denominator, always strictly positive.
    denominator: i128,
}

impl ExactRationalReport {
    /// Project one exact rational losslessly.
    pub(crate) fn of(value: Rational) -> Self {
        Self {
            numerator: value.numerator(),
            denominator: value.denominator(),
        }
    }
}
