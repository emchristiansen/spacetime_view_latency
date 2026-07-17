//! The typed latest-progress carried forward through a campaign's successful child completions.

use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::record_id::RecordId;
use crate::plan::schedule::BlockCoordinate;

use super::block_cursor::BlockComplete;

/// The typed high-water mark of a campaign's progress: the last block that fully completed, the last
/// run within it, and the last record and dose whose writer contract returned success. Every field is a
/// typed coordinate or record identity — never a count or a diagnostic string — so a completed campaign
/// (or a campaign that finished executing but failed only to finalize) reports exactly which scheduled
/// work's writer contracts returned success. Holding these identities is evidence the writer contract
/// returned success for those records; it is not a claim of physical crash persistence.
///
/// It advances only through [`Self::record_block`], which reads a [`BlockComplete`] — itself minted
/// only by a block exhausting its two runs — so progress can only be moved forward by a real child
/// completion, never fabricated. [`Self::before_execution`] is the explicit pre-execution state: no
/// block, run, record, or dose yet.
#[derive(Debug)]
pub(crate) struct LatestProgress {
    last_block: Option<BlockCoordinate>,
    last_run: Option<RunCoordinate>,
    last_successful_record: Option<RecordId>,
    last_durable_dose: Option<DoseIndex>,
}

impl LatestProgress {
    /// The explicit pre-execution progress state, before any block has completed: no block, run,
    /// record, or dose recorded yet. Named for the state it represents rather than obtained from a
    /// blanket default. `pub(in crate::campaign)` so only a campaign cursor starts progress.
    pub(in crate::campaign) fn before_execution() -> Self {
        Self {
            last_block: None,
            last_run: None,
            last_successful_record: None,
            last_durable_dose: None,
        }
    }

    /// Advance the high-water mark to a freshly completed block. Reads the block's own last run,
    /// record, and dose from its completion evidence. `pub(in crate::campaign)` so only a campaign
    /// cursor's completion transition advances progress.
    pub(in crate::campaign) fn record_block(&mut self, done: &BlockComplete) {
        self.last_block = Some(done.block());
        self.last_run = Some(done.last_run().clone());
        self.last_successful_record = Some(done.last_successful_record());
        self.last_durable_dose = Some(done.last_durable_dose());
    }

    /// The last block that fully completed, if any.
    pub(crate) fn last_block(&self) -> Option<BlockCoordinate> {
        self.last_block
    }

    /// The last run that fully completed, if any.
    pub(crate) fn last_run(&self) -> Option<&RunCoordinate> {
        self.last_run.as_ref()
    }

    /// The last record whose writer contract returned success across all completed blocks, if any.
    pub(crate) fn last_successful_record(&self) -> Option<RecordId> {
        self.last_successful_record
    }

    /// The last dose whose observation write's writer contract returned success across all completed
    /// blocks, if any.
    pub(crate) fn last_durable_dose(&self) -> Option<DoseIndex> {
        self.last_durable_dose
    }
}
