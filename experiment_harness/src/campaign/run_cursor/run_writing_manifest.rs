//! The initial run state: writing the run's once-per-run immutable manifest record before any dose.

use crate::campaign::run_frontier::RunFrontier;
use crate::campaign::run_stage::RunStage;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::observation_sink::ObservationSink;

use super::RunDosing;
use super::RunIncomplete;
use super::RunManifestStep;

/// A run that owns the sink but has not yet written its manifest. The only way to obtain the dose
/// ladder is [`Self::write_manifest`]: dose 1 is unreachable until the manifest write's contract
/// returns success, so no observation can precede the manifest record.
pub(crate) struct RunWritingManifest {
    sink: ObservationSink,
    coord: RunCoordinate,
}

impl RunWritingManifest {
    /// Begin a run in its manifest-writing state, taking ownership of the campaign's sink and the
    /// run's coordinate. `pub(in crate::campaign)` so only a block cursor starts a run.
    pub(in crate::campaign) fn begin(sink: ObservationSink, coord: RunCoordinate) -> Self {
        Self { sink, coord }
    }

    /// Write the run's manifest record. The manifest's own run coordinate must match this run — a
    /// mismatch is a wiring bug and fails loud. On a write whose contract returns success, advance to
    /// [`RunDosing`] carrying the returned manifest receipt; on a failure, stop at a
    /// [`RunStage::WritingManifest`] frontier that records the attempted record (if any) and the sink's
    /// retained poison.
    pub(in crate::campaign) fn write_manifest(
        mut self,
        manifest: &ValidatedRunManifest,
    ) -> RunManifestStep {
        assert_eq!(
            manifest.run_coordinate(),
            self.coord,
            "the manifest's run coordinate must match the active run's coordinate"
        );
        match self.sink.write_manifest(manifest) {
            Ok(receipt) => {
                RunManifestStep::Dosing(RunDosing::begin(self.sink, self.coord, receipt))
            }
            Err(error) => {
                let frontier = RunFrontier::new(
                    self.coord,
                    RunStage::WritingManifest,
                    None,
                    None,
                    error.attempted_record(),
                    self.sink.poisoned().cloned(),
                );
                RunManifestStep::Incomplete(RunIncomplete::new(self.sink, frontier))
            }
        }
    }
}
