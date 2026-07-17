//! The initial run state: writing the run's once-per-run immutable manifest record before any dose.

use crate::campaign::run_frontier::RunFrontier;
use crate::campaign::sink_write_stage::SinkWriteStage;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::observation_sink::ObservationSink;

use super::RunExecutionStopped;
use super::RunManifestStep;
use super::RunSeedingBackground;

/// A run that owns the sink but has not yet written its manifest. The only way to enter the pre-dose
/// preparation phases is [`Self::write_manifest`]: the warm-up phase ([`RunSeedingBackground`]) — and so
/// eventually dose 1 — is unreachable until the manifest write's contract returns success, so no effect or
/// observation can precede the manifest record.
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
    /// mismatch is a wiring bug and fails loud. On a write whose contract returns success, advance to the
    /// warm-up phase ([`RunSeedingBackground`]) carrying the returned manifest receipt; on a failure, stop
    /// at a [`RunExecutionStopped`] carrying a
    /// [`RunStage::WritingManifest`](crate::campaign::run_stage::RunStage::WritingManifest) frontier (built
    /// from the typed [`SinkWriteStage`] input) that records the
    /// attempted record (if any) and the sink's retained poison — the run's linear cleanup owner then
    /// settles it.
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
                RunManifestStep::Seeding(RunSeedingBackground::new(self.sink, self.coord, receipt))
            }
            Err(error) => {
                let frontier = RunFrontier::stopped_by_sink_write(
                    self.coord,
                    SinkWriteStage::WritingManifest,
                    None,
                    None,
                    error.attempted_record(),
                    self.sink.poisoned().cloned(),
                );
                RunManifestStep::Stopped(RunExecutionStopped::new(self.sink, frontier))
            }
        }
    }
}
