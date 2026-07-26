//! One channel's reduced statistic `S` at one scale point.

use anyhow::{ensure, Result};
use serde::{Serialize, Serializer};

use crate::analysis::stats::rational::Rational;

/// The spec's `S`: one channel's cell statistic at one scale point, exact and strictly positive.
///
/// Finiteness is structural — [`Rational`] is an `i128/i128` pair in lowest terms with a strictly
/// positive denominator, so no infinity or NaN can be represented at all. Strict positivity is the
/// one property that needs proving, and [`Self::validated`] is the sole constructor, so a
/// nonpositive statistic cannot exist rather than being checked for at each of the several places
/// that consume one.
///
/// That matters because the endpoint factor is `T = S_last / S_first`: a zero `S_first` would make
/// the estimand undefined, and a negative one would let a nonsense ratio satisfy the flat inequality
/// and be reported as flat. The spec's rule — a missing, non-finite, or nonpositive statistic
/// invalidates its block and can never be counted as flat — is therefore enforced here, at the only
/// door into the type.
///
/// Nonpositivity is genuinely anomalous rather than the flat case: the saturated channel's statistic
/// is a queue service time and the other three are durations, all structurally above zero, so
/// rejecting them costs no power against the flat hypothesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CellStatistic(Rational);

impl CellStatistic {
    /// Admit a reduced channel statistic, failing loud unless it is strictly positive.
    pub(crate) fn validated(value: Rational) -> Result<Self> {
        ensure!(
            value.numerator() > 0,
            "a channel cell statistic must be strictly positive; got {}/{}",
            value.numerator(),
            value.denominator(),
        );
        Ok(Self(value))
    }

    /// The exact value, for the endpoint-factor arithmetic that consumes it.
    pub(crate) fn get(self) -> Rational {
        self.0
    }
}

impl Serialize for CellStatistic {
    /// Serialized as the exact `[numerator, denominator]` pair rather than a float, so the ledger
    /// retains the value classification actually used. Floating point is display-only throughout
    /// this contract.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq([self.0.numerator(), self.0.denominator()].iter())
    }
}

#[cfg(test)]
mod tests;
