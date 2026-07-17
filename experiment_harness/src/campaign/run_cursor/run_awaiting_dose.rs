//! The run state awaiting one specific dose's observation before drawing the next dose.

use std::array::IntoIter;

use crate::campaign::run_frontier::RunFrontier;
use crate::campaign::run_stage::RunStage;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::dose_observation::DoseObservation;
use crate::observation::observation_sink::ManifestWriteReceipt;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_id::RecordId;
use crate::params::NUM_DOSES_USIZE;

use super::RunDosing;
use super::RunIncomplete;
use super::RunObservationStep;

/// A run that has drawn a specific `active` dose from its ladder and awaits that dose's observation. The
/// only transition is [`Self::write_observation`], which is bound to `active`: the observation's run
/// coordinate, manifest reference, and dose must all match the active run and dose, so a mismatched or
/// out-of-order observation is a wiring bug and fails loud.
///
/// `last_record` is a required [`RecordId`] (see [`RunDosing`]); `last_dose` is `None` while awaiting
/// the first dose and carries the previous dose otherwise.
pub(crate) struct RunAwaitingDose {
    sink: ObservationSink,
    coord: RunCoordinate,
    manifest: ManifestWriteReceipt,
    doses: IntoIter<DoseIndex, NUM_DOSES_USIZE>,
    last_record: RecordId,
    last_dose: Option<DoseIndex>,
    active: DoseIndex,
}

impl RunAwaitingDose {
    /// Await the drawn `active` dose's observation. `pub(super)` so only a run-cursor transition (drawing
    /// the dose) — not a campaign sibling — supplies the private dose iterator.
    pub(super) fn new(
        sink: ObservationSink,
        coord: RunCoordinate,
        manifest: ManifestWriteReceipt,
        doses: IntoIter<DoseIndex, NUM_DOSES_USIZE>,
        last_record: RecordId,
        last_dose: Option<DoseIndex>,
        active: DoseIndex,
    ) -> Self {
        Self {
            sink,
            coord,
            manifest,
            doses,
            last_record,
            last_dose,
            active,
        }
    }

    /// Write the active dose's observation. The observation is bound to the active run and dose: its run
    /// coordinate must equal this run's, its manifest reference must equal this run's manifest receipt's,
    /// and its dose must equal the drawn `active` dose — mismatches are wiring bugs and fail loud. On a
    /// write whose contract returns success, return to [`RunDosing`] with the freshly written record and
    /// dose as the new last-success markers; on a failure, stop at a [`RunStage::Dosing`] frontier that
    /// carries the prior last-success markers, the attempted record (if any), and the sink's poison.
    pub(in crate::campaign) fn write_observation(
        mut self,
        observation: &DoseObservation,
    ) -> RunObservationStep {
        assert_eq!(
            observation.run_coordinate(),
            self.coord,
            "the observation's run coordinate must match the active run's coordinate"
        );
        assert_eq!(
            observation.manifest_ref(),
            self.manifest.reference(),
            "the observation's manifest reference must match the active run's manifest receipt"
        );
        assert_eq!(
            observation.dose(),
            self.active,
            "the observation's dose must match the drawn active dose"
        );
        match self.sink.write_observation(observation) {
            Ok(receipt) => {
                let last_record = receipt.record();
                let last_dose = Some(receipt.dose());
                RunObservationStep::Dosing(RunDosing::resume(
                    self.sink,
                    self.coord,
                    self.manifest,
                    self.doses,
                    last_record,
                    last_dose,
                ))
            }
            Err(error) => {
                let frontier = RunFrontier::new(
                    self.coord,
                    RunStage::Dosing(self.active),
                    Some(self.last_record),
                    self.last_dose,
                    error.attempted_record(),
                    self.sink.poisoned().cloned(),
                );
                RunObservationStep::Incomplete(RunIncomplete::new(self.sink, frontier))
            }
        }
    }
}
