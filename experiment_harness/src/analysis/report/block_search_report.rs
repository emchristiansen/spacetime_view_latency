//! The report projection of one arm block's complete search result: fit bound to convergence.

use serde::Serialize;

use crate::analysis::beta::block_search_outcome::BlockSearchOutcome;
use crate::analysis::report::block_convergence_report::BlockConvergenceReport;
use crate::analysis::report::block_fit_report::BlockFitReport;

/// The report projection of one block's [`BlockSearchOutcome`]: the authoritative fit outcome and the
/// complete convergence record of the search that produced it. The analysis-domain
/// [`BlockSearchOutcome`](crate::analysis::beta::block_search_outcome::BlockSearchOutcome) already proved
/// the selected basin's candidate equals the fit's candidate, so this projection merely renders that
/// witnessed pair; it never re-derives selection.
#[derive(Debug, Serialize)]
pub(crate) struct BlockSearchReport {
    /// The authoritative per-block fit outcome.
    fit: BlockFitReport,
    /// The complete convergence record of the search that produced the fit.
    convergence: BlockConvergenceReport,
}

impl BlockSearchReport {
    /// Project one block's complete search outcome.
    pub(crate) fn of(outcome: &BlockSearchOutcome) -> Self {
        Self {
            fit: BlockFitReport::of(outcome.fit()),
            convergence: BlockConvergenceReport::of(outcome.convergence()),
        }
    }
}
