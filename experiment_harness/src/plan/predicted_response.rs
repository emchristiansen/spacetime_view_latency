//! Preregistered predicted response.

use crate::plan::growth_regime::GrowthRegime;

/// The preregistered prediction for an arm/regime cell.
///
/// Typed as a predicted response bound to the regime it is predicted in, not a
/// generic exponent (spec: "Classification"). Always *derived* from a
/// [`crate::plan::cell::Cell`], never stored beside it. These are falsifiable
/// hypotheses; contradictory measured results are recorded unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PredictedResponse {
    /// Predicted `Flat-equivalent` in the named regime.
    FlatIn(GrowthRegime),
    /// Predicted `Increasing` in the named regime.
    IncreasingIn(GrowthRegime),
}
