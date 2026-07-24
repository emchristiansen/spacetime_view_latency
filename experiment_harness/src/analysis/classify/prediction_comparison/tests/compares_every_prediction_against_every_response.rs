//! Every (predicted, observed) pair maps to its preregistered comparison: a matching definite class is
//! Confirmed, an opposite definite class is Contradicted, and an Inconclusive observation is
//! Inconclusive regardless of the prediction.

use crate::analysis::classify::prediction_comparison::PredictionComparison;
use crate::analysis::classify::response_class::ResponseClass;
use crate::plan::growth_regime::GrowthRegime;
use crate::plan::predicted_response::PredictedResponse;

#[test]
fn compares_every_prediction_against_every_response() {
    // The comparison is over the response *class*; the prediction's regime does not enter it, so a fixed
    // regime exercises the whole table.
    let flat = PredictedResponse::FlatIn(GrowthRegime::UnrelatedGrowth);
    let increasing = PredictedResponse::IncreasingIn(GrowthRegime::UnrelatedGrowth);

    let table = [
        (flat, ResponseClass::FlatEquivalent, PredictionComparison::Confirmed),
        (flat, ResponseClass::Increasing, PredictionComparison::Contradicted),
        (flat, ResponseClass::Decreasing, PredictionComparison::Contradicted),
        (flat, ResponseClass::Inconclusive, PredictionComparison::Inconclusive),
        (increasing, ResponseClass::Increasing, PredictionComparison::Confirmed),
        (increasing, ResponseClass::FlatEquivalent, PredictionComparison::Contradicted),
        (increasing, ResponseClass::Decreasing, PredictionComparison::Contradicted),
        (increasing, ResponseClass::Inconclusive, PredictionComparison::Inconclusive),
    ];

    for (predicted, observed, expected) in table {
        assert_eq!(
            PredictionComparison::compare(predicted, observed),
            expected,
            "prediction {predicted:?} against observed {observed:?} must be {expected:?}"
        );
    }
}
