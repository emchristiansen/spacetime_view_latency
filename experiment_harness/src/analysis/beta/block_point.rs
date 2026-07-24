//! One `(N, y)` fit point of an arm block's ten-dose ladder.

/// A single fit point for the secondary descriptor: an arm dose's cumulative logical cardinality `N`
/// paired with its R-1 median round-trip latency `y`, both in `f64`. The secondary fit is inherently
/// real-valued, so these carry the primary evidence's underlying integer magnitudes cast once into
/// `f64` at the descriptor boundary — never routed back into a classification decision.
///
/// The fields are sealed with no arbitrary-`f64` minter: production points are built from the integer
/// `logical_n`/`median_nanos` of a trusted dose ([`Self::from_measurements`]), which are structurally
/// finite with `N > 0` and `y ≥ 0`; fractional synthetic fixtures use the `#[cfg(test)]`
/// [`Self::from_synthetic`]. Both funnel through the single private [`Self::checked`] mint boundary,
/// which asserts the finiteness/positivity invariant, so a `BlockPoint` with a NaN/∞ coordinate, a
/// non-positive `N`, or a negative latency is unrepresentable.
///
/// `pub(super)` throughout — points are built and read only within the `beta` module (the descriptor's
/// ladder assembly and the constrained-fit solver), never across the crate.
#[derive(Debug, Clone, Copy)]
pub(super) struct BlockPoint {
    /// The cumulative logical x-axis count `N = dose · BATCH_SIZE` of this dose (finite, `> 0`).
    n: f64,
    /// The R-1 median round-trip latency `y = L(N)` at this dose, in nanoseconds (finite, `≥ 0`).
    y: f64,
}

impl BlockPoint {
    /// The single mint boundary: bind a point only after asserting its invariant (finite `N > 0` and
    /// finite `y ≥ 0`). Both public constructors delegate here, so the invariant is a property of every
    /// `BlockPoint` in existence rather than of caller discipline.
    fn checked(n: f64, y: f64) -> Self {
        assert!(
            n.is_finite() && n > 0.0,
            "a ladder point's logical cardinality N must be finite and strictly positive, got {n}"
        );
        assert!(
            y.is_finite() && y >= 0.0,
            "a ladder point's median latency y must be finite and nonnegative, got {y}"
        );
        Self { n, y }
    }

    /// Build a production ladder point from a trusted dose's integer logical cardinality and median
    /// latency. The `u64`/`u128` magnitudes are structurally finite and nonnegative, and a run's
    /// `logical_n` is `dose · BATCH_SIZE ≥ BATCH_SIZE > 0`; [`Self::checked`] asserts this holds.
    pub(super) fn from_measurements(logical_n: u64, median_nanos: u128) -> Self {
        Self::checked(logical_n as f64, median_nanos as f64)
    }

    /// Build a synthetic ladder point from fractional coordinates, for deterministic fit proofs whose
    /// power-law fixtures are not integer-valued. `#[cfg(test)]` so it never widens the production API;
    /// still routed through [`Self::checked`], so synthetic fixtures cannot smuggle in an invalid point.
    #[cfg(test)]
    pub(super) fn from_synthetic(n: f64, y: f64) -> Self {
        Self::checked(n, y)
    }

    /// The cumulative logical x-axis count `N`.
    pub(super) fn n(self) -> f64 {
        self.n
    }

    /// The R-1 median latency `y = L(N)`.
    pub(super) fn y(self) -> f64 {
        self.y
    }
}
