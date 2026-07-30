//! The candidate within-cell sample counts §569 evaluates.

use serde::Serialize;

/// One candidate `W`. Exactly the seven values §569 names, and no others.
///
/// **An enum rather than a validated integer**, so an eighth candidate is unrepresentable rather than
/// merely unwritten. The decision rule is stated over this exact set; a report that quietly evaluated
/// `W = 750` would be answering a question the spec did not ask, and Control would have to notice
/// that from the numbers rather than from the types.
///
/// [`Self::ALL`] is in ascending order, which is the order the report emits, so a reader comparing
/// candidates reads them in the order the rule discusses them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum CandidateWindow {
    W10,
    W20,
    W50,
    W100,
    W200,
    W500,
    W1000,
}

impl CandidateWindow {
    /// Every candidate `W`, ascending — the whole set the decision rule is stated over.
    pub(crate) const ALL: [CandidateWindow; 7] = [
        CandidateWindow::W10,
        CandidateWindow::W20,
        CandidateWindow::W50,
        CandidateWindow::W100,
        CandidateWindow::W200,
        CandidateWindow::W500,
        CandidateWindow::W1000,
    ];

    /// This candidate's sample count.
    pub(crate) fn get(self) -> usize {
        match self {
            CandidateWindow::W10 => 10,
            CandidateWindow::W20 => 20,
            CandidateWindow::W50 => 50,
            CandidateWindow::W100 => 100,
            CandidateWindow::W200 => 200,
            CandidateWindow::W500 => 500,
            CandidateWindow::W1000 => 1_000,
        }
    }
}

#[cfg(test)]
mod tests;
