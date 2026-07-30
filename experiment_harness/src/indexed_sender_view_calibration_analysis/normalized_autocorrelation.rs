//! The normalized lag-1 coefficient, or the explicit reason it is undefined.

use serde::Serialize;

use crate::analysis::finite_f64::FiniteF64;

/// The lag-1 autocorrelation coefficient as a **total** sum type.
///
/// **Never a default, never a NaN, never silently omitted.** A constant series has zero energy on one
/// or both sides of the lag, so the coefficient is a `0/0` form — genuinely undefined, not zero.
/// Reporting `0.0` there would say "no lag dependence", which is a claim about a series that cannot
/// support one; omitting the field would leave a reader unable to tell an undefined coefficient from
/// a missing computation. The undefined case is therefore its own variant, exactly as the campaign's
/// [`AutocorrelationReport`](crate::analysis::report::autocorrelation_report::AutocorrelationReport)
/// does it.
///
/// The defined case crosses the lossy boundary through [`FiniteF64`] because the normalization
/// involves a square root and so has no exact rational form. That loss is confined to *this* value:
/// the exact numerator and both exact energy terms are retained separately by
/// [`LagAutocorrelation`](super::lag_autocorrelation::LagAutocorrelation), so a reader can recompute
/// the coefficient at any precision, or check this one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub(crate) enum NormalizedAutocorrelation {
    /// Both energy terms are nonzero; the coefficient is a finite value in `[-1, 1]`.
    Defined { lag1: FiniteF64 },
    /// At least one energy term is zero, so the coefficient is a `0/0` form.
    UndefinedZeroVariance,
}
