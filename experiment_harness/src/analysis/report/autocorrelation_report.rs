//! The report projection of a cell's lag-1 autocorrelation diagnostic.

use serde::Serialize;

use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::finite_f64::FiniteF64;

/// The lag-1 autocorrelation of a cell's collection-order-sorted primary `T_block` totals (spec: computed
/// "only as a report-boundary floating-point diagnostic ... it never affects classification"). It is a
/// typed sum type, never a NaN/±∞ sentinel: the coefficient is defined only when the series has nonzero
/// variance, so a zero-variance series is the explicit [`UndefinedZeroVariance`](Self::UndefinedZeroVariance)
/// variant rather than a `0/0` float.
#[derive(Debug, Serialize)]
pub(crate) enum AutocorrelationReport {
    /// The series has nonzero variance; the lag-1 coefficient is a finite value.
    Defined {
        /// The lag-1 autocorrelation coefficient.
        lag1: FiniteF64,
    },
    /// The series has zero variance, so the lag-1 autocorrelation is undefined (a `0/0` form).
    UndefinedZeroVariance,
}

impl AutocorrelationReport {
    /// Project the centered biased lag-1 autocorrelation of a cell's primary `T_block` totals. One input —
    /// the exact evidence, whose `arm_minus_control_totals` are already in schedule-proven collection order
    /// (the same series the sibling plot renders), rendered to `f64` at this report boundary. With the
    /// full-series mean `x̄`, the coefficient is `Σ(i=0..n−2)(xᵢ−x̄)(xᵢ₊₁−x̄) / Σ(i=0..n−1)(xᵢ−x̄)²` — the
    /// *biased* normalization, whose denominator sums all `n` centered squares. The ratio is scale-free, so
    /// the nanosecond totals give the same value as any rescaling. A zero denominator (a constant series has
    /// no variance) is the typed [`UndefinedZeroVariance`](Self::UndefinedZeroVariance), never a `0/0`
    /// sentinel; otherwise the ratio is finite (`|r₁| ≤ 1` for the biased estimator) and minted through the
    /// finite boundary.
    pub(crate) fn of(evidence: &CellEvidence) -> Self {
        // The evidence totals are already collection-order-sorted (CellEvidence's contract), matching the
        // plot's series exactly; render each exact rational to f64 only here at the report boundary.
        let totals: Vec<f64> = evidence
            .arm_minus_control_totals()
            .iter()
            .map(|total| total.to_f64())
            .collect();
        let n = totals.len();
        let mean = totals.iter().sum::<f64>() / n as f64;
        // Biased normalization: the denominator sums all n centered squares, not just the n−1 lag terms.
        let denominator: f64 = totals
            .iter()
            .map(|value| {
                let centered = value - mean;
                centered * centered
            })
            .sum();
        // Numerator: the n−1 adjacent centered products (lag-1 pairs), against the same full-series mean.
        let numerator: f64 = (0..n - 1)
            .map(|i| (totals[i] - mean) * (totals[i + 1] - mean))
            .sum();
        // A constant series has zero variance, so the coefficient is the typed undefined form, not 0/0.
        if denominator == 0.0 {
            return Self::UndefinedZeroVariance;
        }
        Self::Defined {
            lag1: FiniteF64::new(numerator / denominator),
        }
    }
}
