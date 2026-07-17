//! The schedule/ladder coordinate of one dose observation.

use serde::Serialize;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;

/// Where a dose observation sits: the run it belongs to, its 1-based ladder index, the driving
/// role's canonical tag, and the cumulative **logical** x-axis count (`dose * BATCH_SIZE`). All
/// fields are derived from the run coordinate, the resolved dataset, and the dose batch, so no
/// caller supplies an arbitrary dose, role, or x-axis value.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct DoseCoordinate {
    run: RunCoordinate,
    dose: DoseIndex,
    driving_role_tag: &'static str,
    logical_n: u64,
}

impl DoseCoordinate {
    /// Assemble the coordinate from the run, the resolved dataset, and the dose batch.
    pub(crate) fn new(run: RunCoordinate, dataset: &CampaignDataset, batch: &DoseBatch) -> Self {
        Self {
            run,
            dose: batch.dose(),
            driving_role_tag: dataset.driving_role_tag(),
            logical_n: batch.cumulative_driving_rows(),
        }
    }

    /// This observation's ladder index.
    pub(crate) fn dose(&self) -> DoseIndex {
        self.dose
    }
}
