//! The linear/sublinear consistency label of a population β interval.

/// The preregistered consistency label of a population β confidence interval (spec: the descriptor "may
/// emit a ... linear/sublinear consistency label"). The interval is linear-consistent when it lies
/// wholly within the closed band `[0.8, 1.2]`, sublinear-consistent when it lies wholly within the open
/// band `(0.2, 0.8)`, and otherwise unlabeled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BetaLabel {
    /// The whole interval lies within `[0.8, 1.2]`: consistent with a linear `L(N) ∝ N`.
    LinearConsistent,
    /// The whole interval lies within `(0.2, 0.8)`: consistent with a sublinear `L(N) ∝ N^β`, `β < 1`.
    SublinearConsistent,
    /// The interval fits neither band wholly: no consistency label is claimed.
    Unlabeled,
}

impl BetaLabel {
    /// The inclusive lower/upper bounds of the linear-consistency band `[0.8, 1.2]`.
    const LINEAR_LOWER: f64 = 0.8;
    const LINEAR_UPPER: f64 = 1.2;
    /// The exclusive lower/upper bounds of the sublinear-consistency band `(0.2, 0.8)`.
    const SUBLINEAR_LOWER: f64 = 0.2;
    const SUBLINEAR_UPPER: f64 = 0.8;

    /// Label a population interval `[lo, hi]` by which preregistered band, if any, contains it wholly.
    /// The linear band is closed and the sublinear band is open, so an endpoint of exactly `0.8` is
    /// linear-consistent, never sublinear-consistent; the two bands are checked in that order and are
    /// disjoint, so at most one applies.
    pub(crate) fn classify(interval_lo: f64, interval_hi: f64) -> Self {
        if interval_lo >= Self::LINEAR_LOWER && interval_hi <= Self::LINEAR_UPPER {
            Self::LinearConsistent
        } else if interval_lo > Self::SUBLINEAR_LOWER && interval_hi < Self::SUBLINEAR_UPPER {
            Self::SublinearConsistent
        } else {
            Self::Unlabeled
        }
    }
}
