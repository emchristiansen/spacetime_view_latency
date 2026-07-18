//! Exact reduced rational arithmetic for the primary estimator.

use std::cmp::Ordering;

/// An exact rational number stored in lowest terms with a strictly positive denominator, so every
/// value has a unique canonical `(num, den)` and structural equality coincides with numeric equality.
///
/// The primary estimator (the Theil–Sen median of 45 pairwise slopes, the order-statistic median
/// confidence interval, and the δ margin) is exact integer/rational arithmetic end to end — no
/// floating point enters any classification decision. The `i128` components carry the campaign's
/// magnitudes (nanosecond latencies of order `1e7`–`1e10`, ladder x-values `≤ 1e4`, and differences
/// and pairwise products of these) with many orders of magnitude of headroom; every arithmetic step
/// is a `checked_*` operation that fails loud rather than wrapping, matching the crate's fail-fast
/// policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rational {
    num: i128,
    den: i128,
}

impl Rational {
    /// Construct `num/den` in canonical form: denominator made strictly positive, then both
    /// components divided by their greatest common divisor. A zero denominator is an invariant
    /// violation, not a recoverable input, so it fails loud.
    pub(crate) fn new(num: i128, den: i128) -> Self {
        assert!(den != 0, "a rational's denominator is never zero");
        let (num, den) = if den < 0 {
            (
                num.checked_neg()
                    .expect("rational numerator negation fits i128"),
                den.checked_neg()
                    .expect("rational denominator negation fits i128"),
            )
        } else {
            (num, den)
        };
        let divisor = gcd(num.unsigned_abs(), den.unsigned_abs());
        let divisor = i128::try_from(divisor).expect("a gcd of two i128 magnitudes fits i128");
        Self {
            num: num / divisor,
            den: den / divisor,
        }
    }

    /// The integer `n` as the rational `n/1`.
    pub(crate) fn from_int(n: i128) -> Self {
        Self { num: n, den: 1 }
    }

    /// The additive identity `0/1`.
    pub(crate) fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    /// The exact sum `self + other`.
    pub(crate) fn add(self, other: Self) -> Self {
        let left = self
            .num
            .checked_mul(other.den)
            .expect("rational addition cross term fits i128");
        let right = other
            .num
            .checked_mul(self.den)
            .expect("rational addition cross term fits i128");
        let num = left
            .checked_add(right)
            .expect("rational addition numerator fits i128");
        let den = self
            .den
            .checked_mul(other.den)
            .expect("rational addition denominator fits i128");
        Self::new(num, den)
    }

    /// The exact difference `self - other`.
    pub(crate) fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }

    /// The exact product `self * other`.
    pub(crate) fn mul(self, other: Self) -> Self {
        let num = self
            .num
            .checked_mul(other.num)
            .expect("rational product numerator fits i128");
        let den = self
            .den
            .checked_mul(other.den)
            .expect("rational product denominator fits i128");
        Self::new(num, den)
    }

    /// The additive inverse `-self`.
    pub(crate) fn neg(self) -> Self {
        Self {
            num: self
                .num
                .checked_neg()
                .expect("rational numerator negation fits i128"),
            den: self.den,
        }
    }

    /// The exact quotient `self / k` for a nonzero integer `k`.
    pub(crate) fn div_int(self, k: i128) -> Self {
        assert!(k != 0, "rational division by zero is undefined");
        let den = self
            .den
            .checked_mul(k)
            .expect("rational integer-division denominator fits i128");
        Self::new(self.num, den)
    }

    /// The canonical numerator (sign lives here; denominator is always positive).
    pub(crate) fn numerator(self) -> i128 {
        self.num
    }

    /// The canonical strictly-positive denominator.
    pub(crate) fn denominator(self) -> i128 {
        self.den
    }

    /// A lossy `f64` rendering, for human-readable/JSON *display* only — never a classification input.
    pub(crate) fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl Ord for Rational {
    /// Compare `a/b` and `c/d` (both denominators strictly positive) by the sign of `a*d - c*b`,
    /// evaluated with checked cross-multiplication so an overflow fails loud rather than misordering.
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self
            .num
            .checked_mul(other.den)
            .expect("rational comparison cross term fits i128");
        let right = other
            .num
            .checked_mul(self.den)
            .expect("rational comparison cross term fits i128");
        left.cmp(&right)
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The greatest common divisor of two magnitudes by the Euclidean algorithm. `gcd(0, d) = d`, and a
/// result is clamped to at least `1` so the canonicalizing division in [`Rational::new`] never divides
/// by zero even in the (unreachable, denominator-nonzero) all-zero case.
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.max(1)
}

#[cfg(test)]
mod tests;
