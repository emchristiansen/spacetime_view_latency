//! The frozen preregistered practical-equivalence margin `δ`.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;
use crate::params::{NUM_DOSES_USIZE, REPETITION_BLOCKS};

/// The number of matched-control dose medians a cell's margin is frozen over: one per dose of every
/// run in the complete 30-block cell (30 blocks × 10 doses = 300). Fixed cardinality so the frozen
/// margin's input is a compile-visible census, not a runtime-checked vector. `pub(crate)` so the
/// campaign classifier that assembles the 300 control dose medians shares this single canonical
/// cardinality rather than re-deriving the same `30 × 10` product.
pub(crate) const CONTROL_DOSE_MEDIAN_COUNT: usize = REPETITION_BLOCKS as usize * NUM_DOSES_USIZE;

/// The frozen practical-equivalence margin `δ = 0.20 · median(L_control)` over all 300 matched-control
/// dose medians in the complete 30-block cell (spec: "Classification"). It has latency units like
/// `T_block`, defines practical flatness as the symmetric band `[-δ, +δ]`, and is frozen once — before
/// any validity or response classification — so both the arm response and the control-validity gate use
/// the same `δ`.
///
/// The stored `delta` is an exact [`Rational`]; the field is private so a margin is built only by
/// [`Self::freeze`]. Floating-point rendering exists only for display, never for a classification
/// comparison.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EquivalenceMargin {
    delta: Rational,
}

impl EquivalenceMargin {
    /// Freeze `δ` as one fifth of the exact median of the cell's 300 matched-control dose medians,
    /// taking the exact average of the two central order statistics for the even-sized sample (spec:
    /// exact-rational classifier decision). The sample size is pinned by the array length, so an
    /// off-count input is a compile error rather than a runtime check.
    pub(crate) fn freeze(control_dose_medians: &[Rational; CONTROL_DOSE_MEDIAN_COUNT]) -> Self {
        // `median` takes the exact even-count mean of the two central order statistics for this
        // 300-sample cell, and `div_int(5)` scales by exactly one fifth — both exact rational steps.
        Self {
            delta: median(control_dose_medians).div_int(5),
        }
    }

    /// The exact half-width `δ` of the practical-equivalence band.
    pub(crate) fn delta(self) -> Rational {
        self.delta
    }

    /// The band's lower bound `-δ`.
    pub(crate) fn lower(self) -> Rational {
        self.delta.neg()
    }

    /// The band's upper bound `+δ`.
    pub(crate) fn upper(self) -> Rational {
        self.delta
    }

    /// A lossy millisecond rendering of `δ`, for the machine-readable report only — never a
    /// classification input (spec: the margin "must be reported in milliseconds").
    pub(crate) fn to_millis_f64(self) -> f64 {
        // δ is an exact rational count of nanoseconds; render it in milliseconds for the report only.
        self.delta.to_f64() / 1_000_000.0
    }
}
