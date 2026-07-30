//! The normalized coefficient at one lag, or the explicit reason it is undefined.

use serde::Serialize;

use crate::indexed_sender_view_calibration_analysis::correlation_coefficient::CorrelationCoefficient;

/// The autocorrelation coefficient at whichever lag the containing element reports, as a **total**
/// sum type.
///
/// Deliberately not named for lag 1: the domain runs from 1 to one below the widest candidate, so
/// 998 of the 999 elements are not lag 1, and a lag-1 name would misdescribe them.
///
/// **Never a default, never a NaN, never silently omitted.** A constant series has zero energy on one
/// or both sides of the lag, so the coefficient is a `0/0` form — genuinely undefined, not zero.
/// Reporting `0.0` there would say "no lag dependence", which is a claim about a series that cannot
/// support one; omitting the field would leave a reader unable to tell an undefined coefficient from
/// a missing computation. The undefined case is therefore its own variant, exactly as the campaign's
/// [`AutocorrelationReport`](crate::analysis::report::autocorrelation_report::AutocorrelationReport)
/// does it.
///
/// The defined case crosses the lossy boundary through [`CorrelationCoefficient`] — itself a
/// range-enforcing wrapper over [`FiniteF64`](crate::analysis::finite_f64::FiniteF64) — because the
/// normalization
/// involves a square root and so has no exact rational form. That loss is confined to *this* value:
/// the exact numerator and both exact energy terms are retained separately by
/// [`LagAutocorrelation`](super::lag_autocorrelation::LagAutocorrelation), so a reader can recompute
/// the coefficient at any precision, or check this one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub(crate) enum NormalizedAutocorrelation {
    /// Both energy terms are nonzero, so the coefficient exists. Its `[-1, 1]` range is **enforced
    /// by [`CorrelationCoefficient`]**, not merely asserted here: the exact integer components prove
    /// the mathematical value lies in that range, and that type repairs the rounding of the `f64`
    /// projection while failing loudly on anything larger.
    ///
    /// Named `coefficient` rather than `lag1`: this type is the normalization of *whichever* lag its
    /// containing [`LagAutocorrelation`](super::lag_autocorrelation::LagAutocorrelation) element is
    /// for, and the domain runs to one below the widest candidate. A field spelled `lag1` would be
    /// false on 998 of the 999 elements.
    Defined { coefficient: CorrelationCoefficient },
    /// At least one energy term is zero, so the coefficient is a `0/0` form.
    UndefinedZeroVariance,
}
