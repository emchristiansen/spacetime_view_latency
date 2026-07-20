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
    /// Project the lag-1 autocorrelation of a cell's primary totals. One input — the exact evidence
    /// carrying the 30 `T_block` totals and their collection-order keys — sorted by collection order at the
    /// report boundary; the zero-variance case resolves to the typed undefined variant, never a sentinel.
    pub(crate) fn of(evidence: &CellEvidence) -> Self {
        let _ = evidence;
        todo!("Phase 2: compute lag-1 autocorrelation over collection-order-sorted totals; typed undefined on zero variance")
    }
}
