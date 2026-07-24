//! The report projection of one cell's secondary `L(N)=a+bN^β` descriptor.

use serde::Serialize;

use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::report::beta_population_report::BetaPopulationReport;
use crate::analysis::report::block_search_report::BlockSearchReport;
use crate::params::REPETITION_BLOCKS;

/// The fixed 30-block per-cell sample size as an array length, sized from the single frozen
/// [`REPETITION_BLOCKS`] source rather than a re-typed literal (mirroring
/// [`BetaDescriptor`](crate::analysis::beta::beta_descriptor)'s own `N_BLOCKS`).
const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The report projection of one gated-Increasing cell's [`BetaDescriptor`]: the complete per-block search
/// summary of each of its 30 arm blocks (fit outcome bound to convergence record), and the optional
/// population β interval derived only when every block is identifiable. It is a *mandatory* field of the
/// [`Increasing`](crate::analysis::report::classified_cell_report::ClassifiedCellReport::Increasing)
/// classification branch — never an `Option` on the cell report — so the secondary descriptor exists
/// exactly for a gated Increasing cell and cannot appear for any other. Its own `population` field is the
/// only `Option` here, `Some` only when every block is identifiable.
#[derive(Debug, Serialize)]
pub(crate) struct SecondaryDescriptorReport {
    /// The population β interval and label, present only when all 30 blocks are identifiable.
    population: Option<BetaPopulationReport>,
    /// The 30 per-block search summaries, in schedule-proven collection order — a boxed fixed array so the
    /// 30-block cardinality is a property of the type.
    blocks: Box<[BlockSearchReport; N_BLOCKS]>,
}

impl SecondaryDescriptorReport {
    /// Project one cell's secondary descriptor, including every block's all-basin convergence telemetry.
    pub(crate) fn of(descriptor: &BetaDescriptor) -> Self {
        // Project the 30 per-block search summaries heap-first into the fixed array, mirroring the
        // descriptor's own boxed storage. The population interval is `Some` exactly when every block is
        // identifiable — the source derives that, so this projection cannot introduce a disagreement.
        let blocks: Vec<BlockSearchReport> = descriptor
            .block_search_outcomes()
            .iter()
            .map(BlockSearchReport::of)
            .collect();
        let blocks = blocks
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly N_BLOCKS search outcomes project into exactly N_BLOCKS block reports");
        Self {
            population: descriptor
                .population()
                .as_ref()
                .map(BetaPopulationReport::of),
            blocks,
        }
    }
}

#[cfg(test)]
mod tests;
