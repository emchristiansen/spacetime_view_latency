//! The report projection of one arm block's complete search result: fit bound to convergence.

use serde::Serialize;

use crate::analysis::beta::keyed_block_outcome::KeyedBlockOutcome;
use crate::analysis::report::block_convergence_report::BlockConvergenceReport;
use crate::analysis::report::block_fit_report::BlockFitReport;
use crate::observation::record_seq::RecordSeq;

/// The report projection of one block's [`KeyedBlockOutcome`]: the block's schedule-proven collection-order
/// key, the authoritative fit outcome, and the complete convergence record of the search that produced it.
/// The serialized `collection_order_key` is the explicit join key — it lets a report consumer align this β
/// outcome with the primary temporal point of the same block by key rather than by array position (spec:
/// temporal diagnostics "join ... β outcomes only by the proven `collection_order_key`; no cross-sibling
/// interpretation may depend on positional coincidence"). The analysis-domain
/// [`BlockSearchOutcome`](crate::analysis::beta::block_search_outcome::BlockSearchOutcome) already proved
/// the selected basin's candidate equals the fit's candidate, so this projection merely renders that
/// witnessed pair; it never re-derives selection.
#[derive(Debug, Serialize)]
pub(crate) struct BlockSearchReport {
    /// This block's schedule-proven collection-order key — the explicit key the report join is keyed on.
    collection_order_key: RecordSeq,
    /// The authoritative per-block fit outcome.
    fit: BlockFitReport,
    /// The complete convergence record of the search that produced the fit.
    convergence: BlockConvergenceReport,
}

impl BlockSearchReport {
    /// Project one block's key-tagged complete search outcome, rendering its collection-order key alongside
    /// the fit and convergence.
    pub(crate) fn of(keyed: &KeyedBlockOutcome) -> Self {
        let outcome = keyed.outcome();
        Self {
            collection_order_key: keyed.collection_order_key(),
            fit: BlockFitReport::of(outcome.fit()),
            convergence: BlockConvergenceReport::of(outcome.convergence()),
        }
    }
}
