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
/// The nanosecond magnitudes and the window sizes involved keep both components far inside `i128`,
/// and every arithmetic step that produced them was a `checked_*` operation that fails loud.
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
