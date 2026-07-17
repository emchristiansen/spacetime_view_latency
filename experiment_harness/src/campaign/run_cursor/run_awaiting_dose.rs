//! The run state awaiting one specific dose's observation before drawing the next dose.

use std::array::IntoIter;

use anyhow::Error;

use crate::campaign::effect_stage::EffectStage;
use crate::campaign::run_frontier::RunFrontier;
use crate::campaign::sink_write_stage::SinkWriteStage;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::dose_observation::DoseObservation;
use crate::observation::observation_sink::ManifestWriteReceipt;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_id::RecordId;
use crate::params::NUM_DOSES_USIZE;

use super::RunDosing;
use super::RunExecutionStopped;
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
    /// dose as the new last-success markers; on a failure, stop at a [`RunExecutionStopped`] carrying a
    /// [`RunStage::Dosing`](crate::campaign::run_stage::RunStage::Dosing) frontier (built from the typed
    /// [`SinkWriteStage`] input) with the prior last-success markers, the attempted record (if any), and the
    /// sink's poison — the run's linear cleanup owner then settles it.
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
                let frontier = RunFrontier::stopped_by_sink_write(
                    self.coord,
                    SinkWriteStage::Dosing(self.active),
                    Some(self.last_record),
                    self.last_dose,
                    error.attempted_record(),
                    self.sink.poisoned().cloned(),
                );
                RunObservationStep::Stopped(RunExecutionStopped::new(self.sink, frontier))
            }
        }
    }

    /// Stop the run because a per-dose *server effect* failed — this dose's measured writes or its
    /// post-write correctness/event check — with the dose drawn but its observation not yet written. This
    /// is the effect-failure sibling of [`Self::write_observation`]'s write-failure edge: it consumes the
    /// awaiting-dose state, keeps the identical
    /// [`RunStage::Dosing`](crate::campaign::run_stage::RunStage::Dosing) stage (built from the typed
    /// [`EffectStage`] input) for the active dose and the
    /// original `error` as typed effect evidence, and yields the same inert [`RunExecutionStopped`] carrier
    /// its linear cleanup owner settles — no optional cleanup, no drop-the-cursor escape hatch, no second
    /// settlement path. The last-success markers are the prior ones: the last record and dose whose writer
    /// contracts returned success.
    pub(in crate::campaign) fn stop_execution(self, error: Error) -> RunExecutionStopped {
        let frontier = RunFrontier::stopped_by_effect(
            self.coord,
            EffectStage::Dosing(self.active),
            Some(self.last_record),
            self.last_dose,
            error,
        );
        RunExecutionStopped::new(self.sink, frontier)
    }
}
