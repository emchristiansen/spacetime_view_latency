//! The lifecycle stages at which a run's *sink write* can fail: the manifest or a dose observation.

use crate::dataset::dose_index::DoseIndex;

use super::run_stage::RunStage;

/// The exact subset of [`RunStage`] a *sink write* can fail in — writing the once-per-run manifest
/// record, or writing a dose's observation. [`RunFrontier::stopped_by_sink_write`] takes this rather than
/// an arbitrary [`RunStage`] and widens it internally, so a sink-write frontier tagged with a pre-dose
/// effect stage (`BackgroundSeed`/`InitialSetCheck`) or a cleanup stage (`Disconnecting`/`Teardown`) is
/// unrepresentable.
///
/// [`RunFrontier::stopped_by_sink_write`]: super::run_frontier::RunFrontier::stopped_by_sink_write
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SinkWriteStage {
    /// Writing the run's once-per-run immutable manifest record.
    WritingManifest,
    /// Writing the observation for the dose at the given ladder index.
    Dosing(DoseIndex),
}

impl SinkWriteStage {
    /// Widen to the corresponding [`RunStage`]. `pub(in crate::campaign)` so only a frontier constructor
    /// widens it.
    pub(in crate::campaign) fn stage(self) -> RunStage {
        match self {
            SinkWriteStage::WritingManifest => RunStage::WritingManifest,
            SinkWriteStage::Dosing(dose) => RunStage::Dosing(dose),
        }
    }
}
