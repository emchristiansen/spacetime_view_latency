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
    /// Compare the preregistered prediction with the observed response. The prediction names a definite
    /// class (Flat-equivalent or Increasing) in the cell's regime; the observed response's regime is
    /// fixed by the same cell, so the comparison is over the response *class* alone. An `Inconclusive`
    /// observation neither confirms nor contradicts; a definite observation matching the named class is
    /// `Confirmed`, and any other definite class is `Contradicted`.
    pub(crate) fn compare(predicted: PredictedResponse, observed: ResponseClass) -> Self {
        match (predicted, observed) {
            (_, ResponseClass::Inconclusive) => Self::Inconclusive,
            (PredictedResponse::FlatIn(_), ResponseClass::FlatEquivalent) => Self::Confirmed,
            (PredictedResponse::FlatIn(_), ResponseClass::Increasing | ResponseClass::Decreasing) => {
                Self::Contradicted
            }
            (PredictedResponse::IncreasingIn(_), ResponseClass::Increasing) => Self::Confirmed,
            (
                PredictedResponse::IncreasingIn(_),
                ResponseClass::FlatEquivalent | ResponseClass::Decreasing,
            ) => Self::Contradicted,
        }
    }
}

#[cfg(test)]
mod tests;
