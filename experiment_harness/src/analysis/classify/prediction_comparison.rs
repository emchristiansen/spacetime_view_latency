//! The comparison of a cell's preregistered prediction against its observed response.

use crate::analysis::classify::response_class::ResponseClass;
use crate::plan::predicted_response::PredictedResponse;

/// How a cell's observed [`ResponseClass`] compares with its preregistered
/// [`PredictedResponse`] (spec success criteria: "compares observed results with every preregistered
/// prediction ... explicitly reports contradictions or inconclusive cells"). Emitted only for a cell
/// with a valid control, alongside the observed response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) enum PredictionComparison {
    /// The observed response is the definite class the prediction named.
    Confirmed,
    /// The observed response is a definite class opposite the prediction.
    Contradicted,
    /// The observed response is `Inconclusive`, so the prediction is neither confirmed nor
    /// contradicted.
    Inconclusive,
}

impl PredictionComparison {
    /// Compare the preregistered prediction with the observed response.
    pub(crate) fn compare(_predicted: PredictedResponse, _observed: ResponseClass) -> Self {
        todo!("Phase 2: the complete predicted-versus-observed comparison table")
    }
}
