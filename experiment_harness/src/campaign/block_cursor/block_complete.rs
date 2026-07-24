//! Unforgeable, affine evidence that one repetition block completed both its runs.

use crate::campaign::run_cursor::RunComplete;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::record_id::RecordId;
use crate::plan::schedule::BlockCoordinate;

/// Evidence that one block ran both of its runs to completion: the block coordinate, the last run to
/// complete, and that run's last record and dose whose writer contract returned success. It is built
/// only by consuming the last run's affine [`RunComplete`], so its markers are exactly that run's — not
/// a count or a fabricated coordinate.
///
/// The constructor is `pub(super)`: only the enclosing block cursor module mints one, and only at its
/// `[Run; 2]` exhaustion edge. **Affine, not `Clone`:** a block completion is consumed exactly once (by
/// its campaign, to advance latest progress); duplicating it would let one finished block advance
/// progress twice.
///
/// Holding it is evidence the block's record writes' contracts returned success; it is not a claim of
/// physical crash persistence.
#[derive(Debug)]
pub(crate) struct BlockComplete {
    block: BlockCoordinate,
    last_run: RunCoordinate,
    last_successful_record: RecordId,
    last_durable_dose: DoseIndex,
}

impl BlockComplete {
    /// Mint block-completion evidence from the block coordinate and its last completed run. Consumes the
    /// run's [`RunComplete`] — reading exactly that run's coordinate and last-success markers.
    /// `pub(super)` so only the block cursor module (at its exhaustion edge) constructs one.
    pub(super) fn new(block: BlockCoordinate, last_run: RunComplete) -> Self {
        Self {
            block,
            last_run: last_run.run().clone(),
            last_successful_record: last_run.last_successful_record(),
            last_durable_dose: last_run.last_durable_dose(),
        }
    }

    /// The block that completed.
    pub(crate) fn block(&self) -> BlockCoordinate {
        self.block
    }

    /// The last run of this block to complete.
    pub(crate) fn last_run(&self) -> &RunCoordinate {
        &self.last_run
    }

    /// The last record whose writer contract returned success in this block.
    pub(crate) fn last_successful_record(&self) -> RecordId {
        self.last_successful_record
    }

    /// The last dose whose observation write's contract returned success in this block.
    pub(crate) fn last_durable_dose(&self) -> DoseIndex {
        self.last_durable_dose
    }
}
