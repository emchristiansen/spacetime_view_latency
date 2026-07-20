//! The report projection of one basin refinement's termination cause.

use serde::Serialize;

use crate::analysis::beta::golden_stop::GoldenStop;

/// The report projection of a [`GoldenStop`]: whether the golden-section refinement stopped because its
/// bracket contracted to the frozen tolerance or because the iteration cap was hit. A plain closed enum,
/// so the serialized cause can never be a state the search cannot reach.
#[derive(Debug, Serialize)]
pub(crate) enum GoldenStopReport {
    /// The bracket contracted to at most the frozen tolerance — converged to tolerance.
    BracketWidthReached,
    /// The iteration cap was hit while the bracket was still wider than the tolerance — truncated.
    IterationCapReached,
}

impl GoldenStopReport {
    /// Project the analysis-domain stop cause. Takes the `Copy` [`GoldenStop`] by value — one input.
    pub(crate) fn of(stop: GoldenStop) -> Self {
        let _ = stop;
        todo!("Phase 2: map GoldenStop variants one-to-one")
    }
}
