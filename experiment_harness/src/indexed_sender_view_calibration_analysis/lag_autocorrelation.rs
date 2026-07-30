//! One lag's dependence diagnostic, with its exact components retained.

use serde::Serialize;

use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::normalized_autocorrelation::NormalizedAutocorrelation;

/// One replicate's dependence at **one** lag `k`, reported as its exact components plus a normalized
/// coefficient — never as the coefficient alone.
///
/// §569 weighs autocorrelation, and a bare `r = 0.31` is not reviewable: it hides which normalization
/// produced it, and a reader who disagrees with that choice has no way to recompute. So the
/// definition is laid out in the artifact itself. With `x̄` the mean of the whole series, `n` its
/// length, and `m = n − k` the number of lag-`k` pairs:
///
/// - `numerator = Σ_{i=0}^{m-1} (xᵢ − x̄)(xᵢ₊ₖ − x̄)` — the lag-`k` autocovariance numerator;
/// - `left_energy = Σ_{i=0}^{m-1} (xᵢ − x̄)²` — the `m` terms that pair as `xᵢ`;
/// - `right_energy = Σ_{i=k}^{n-1} (xᵢ − x̄)²` — the `m` terms that pair as `xᵢ₊ₖ`;
/// - `normalized = numerator / √(left_energy · right_energy)`.
///
/// All three components are **exact rationals**, and the mean is exact too — the samples are
/// integers, so `x̄` is a rational and every centred term stays rational. Only the final ratio is
/// floating point, because the square root has no exact rational form.
///
/// **The two energies are retained separately rather than collapsed**, which is what makes the
/// definition checkable. This is the Cauchy–Schwarz normalization, so `|normalized| ≤ 1` always
/// holds; the more common same-series convention divides by the *full* `Σ(xᵢ − x̄)²` instead and would
/// give a slightly different number. Publishing both energies lets a reader see which was used and
/// recompute the other, rather than having to trust a label.
///
/// **This type is one lag, not the whole diagnostic.** §568 asks for "lag dependence/effective
/// information" and §569 for "autocorrelation" — neither names a lag, and lag-1 alone cannot speak to
/// effective information, which is a statement about dependence across lags. Which lags are computed
/// is a separate, explicitly derived decision recorded by
/// [`lag_domain`](super::lag_domain::lag_domain).
///
/// The `pairs` count travels with each element for exactly that reason: at the far end of the domain
/// a coefficient rests on very few pairs, and a reader must be able to see that rather than infer it.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct LagAutocorrelation {
    /// Which lag `k` this element is for.
    lag: usize,
    /// How many lag-`k` pairs the series contains — `n − k`.
    pairs: usize,
    /// The exact lag-`k` autocovariance numerator.
    numerator: ExactRationalReport,
    /// The exact energy of the `m` leading terms.
    left_energy: ExactRationalReport,
    /// The exact energy of the `m` trailing terms.
    right_energy: ExactRationalReport,
    /// The normalized coefficient, or the explicit reason it is undefined.
    normalized: NormalizedAutocorrelation,
}

impl LagAutocorrelation {
    /// Compute one replicate's dependence at lag `k`, over its whole series in issue order.
    ///
    /// Over the *whole* series and never per candidate window: dependence is a property of the run.
    pub(crate) fn of(replicate: &CompleteReplicate, lag: usize) -> Self {
        todo!("exact centred numerator and both energies at this lag, then the normalized ratio")
    }
}

#[cfg(test)]
mod tests;
