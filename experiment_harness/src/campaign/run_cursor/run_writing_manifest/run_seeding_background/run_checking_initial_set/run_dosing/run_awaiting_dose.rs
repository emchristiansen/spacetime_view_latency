//! The run state awaiting one specific dose's observation before drawing the next dose.

use anyhow::Error;

use crate::campaign::effect_stage::EffectStage;
use crate::campaign::run_cursor::RunExecutionStopped;
use crate::campaign::run_cursor::RunObservationStep;
use crate::campaign::run_frontier::RunFrontier;
use crate::campaign::sink_write_stage::SinkWriteStage;
use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::run_dataset::RunDataset;
use crate::observation::dose_evidence::DoseEvidence;
use crate::observation::dose_observation::DoseObservation;

use super::RunDosing;

/// A run that has drawn a specific `active` dose from its ladder and awaits that dose's observation. It
/// **owns the entire prior [`RunDosing`]** plus the drawn `active`: this is the paused mid-dose form of the
/// same cycle, not a reconstruction of it. As a **child module** of [`RunDosing`]'s module, it reaches the
/// nested dosing's private sink, context, and progress directly — so [`Self::write_observation`] assembles
/// the observation from the nested dosing's own context and `active`, performs the real sink write, consumes
/// the affine
/// [`DoseWriteReceipt`](crate::observation::observation_sink::DoseWriteReceipt) internally, updates *that
/// same* dosing's private markers, and returns it. There is no observation, coordinate, dose, or progress
/// value passed across a module boundary that could carry a foreign run, and no `RunDosing` write/advance
/// method a sibling could call to drive one run's dosing with another's observation.
///
/// The only constructor is [`Self::new`], `pub(super)` to [`RunDosing`]'s module, so construction is
/// confined to the `run_dosing` subtree ({`run_dosing`, `run_awaiting_dose`}); the implemented call site is
/// [`RunDosing::next_dose`](super::RunDosing), which draws `active` from the dosing's own iterator, so the
/// pairing is always of a dosing with the dose its own ladder yielded. No module outside that subtree can
/// pair them. (`pub(super)` reaches the parent's descendants, so this is a subtree seal, not single-caller
/// enforcement.)
pub(crate) struct RunAwaitingDose {
    dosing: RunDosing,
    active: DoseIndex,
}

impl RunAwaitingDose {
    /// Pair a dosing with the dose it just drew. `pub(super)` — where `super` is [`RunDosing`]'s module —
    /// confining construction to the `run_dosing` subtree; the implemented call site is
    /// [`RunDosing::next_dose`](super::RunDosing), pairing it only with an `active` drawn from that same
    /// dosing's iterator.
    pub(super) fn new(dosing: RunDosing, active: DoseIndex) -> Self {
        Self { dosing, active }
    }

    /// The dose this state has drawn and awaits the observation for. Exposed so the run driver can apply
    /// the inter-dose pacing and issue that exact dose's measured batch — always from the drawn dose, never
    /// from parallel bookkeeping. `pub(in crate::campaign)` so only the run driver reads it.
    pub(in crate::campaign) fn active(&self) -> DoseIndex {
        self.active
    }

    /// The bound run context (manifest + resolved dataset) of the nested dosing, borrowed so the run driver
    /// can issue this dose's measured batch and derive its expected set from this run's own dataset — never a
    /// parallel value. `pub(in crate::campaign)` so only the run driver reads it.
    pub(in crate::campaign) fn run_dataset(&self) -> &RunDataset {
        &self.dosing.context
    }

    /// Write the active dose's observation, assembled from the nested dosing's own context and drawn
    /// `active` dose plus the measured [`DoseEvidence`] the driver returns. The observation is written
    /// through the nested dosing's *own* sink; on success the affine dose-write receipt is consumed here to
    /// advance *that same* dosing's private last-record/last-dose markers, and the advanced dosing is
    /// returned to [`RunDosing`]. Nothing crosses a module boundary — no observation, receipt, record, or
    /// dose is handed to a `RunDosing` method — so a dosing can only ever be advanced by its own observation.
    ///
    /// On a write whose contract returns success, return to [`RunDosing`] with the advanced markers; on a
    /// failure, stop at a [`RunExecutionStopped`] carrying a
    /// [`RunStage::Dosing`](crate::campaign::run_stage::RunStage::Dosing) frontier (built from the typed
    /// [`SinkWriteStage`] input) with the prior last-success markers, the attempted record (if any), and the
    /// sink's poison — the run's linear cleanup owner then settles it. The frontier coordinate is read from
    /// the nested context, not a separate field.
    pub(in crate::campaign) fn write_observation(
        self,
        evidence: DoseEvidence,
    ) -> RunObservationStep {
        let RunAwaitingDose { mut dosing, active } = self;
        let batch = DoseBatch::new(dosing.context.dataset(), active);
        let observation = DoseObservation::assemble(&dosing.context, &batch, evidence);
        match dosing.sink.write_observation(&observation) {
            Ok(receipt) => {
                // Consume the affine receipt against this run's own progress: the dose ladder advances only
                // on the actual written record and dose, and the receipt never escapes this module.
                dosing.last_record = receipt.record();
                dosing.last_dose = Some(receipt.dose());
                RunObservationStep::Dosing(dosing)
            }
            Err(error) => {
                let frontier = RunFrontier::stopped_by_sink_write(
                    dosing.context.manifest().run_coordinate(),
                    SinkWriteStage::Dosing(active),
                    Some(dosing.last_record),
                    dosing.last_dose,
                    error.attempted_record(),
                    dosing.sink.poisoned().cloned(),
                );
                RunObservationStep::Stopped(RunExecutionStopped::new(dosing.sink, frontier))
            }
        }
    }

    /// Stop the run because a per-dose *server effect* failed — this dose's measured writes or its
    /// post-write correctness/event check — with the dose drawn but its observation not yet written. This
    /// is the effect-failure sibling of [`Self::write_observation`]'s write-failure edge: it consumes the
    /// awaiting-dose state, keeps the identical
    /// [`RunStage::Dosing`](crate::campaign::run_stage::RunStage::Dosing) stage (built from the typed
    /// [`EffectStage`] input) for the active dose and the original `error` as typed effect evidence, and
    /// yields the same inert [`RunExecutionStopped`] carrier its linear cleanup owner settles — built and
    /// consumed directly from the nested dosing, with no optional cleanup, no drop-the-cursor escape hatch,
    /// and no second settlement path. The last-success markers are the prior ones the nested dosing holds.
    pub(in crate::campaign) fn stop_execution(self, error: Error) -> RunExecutionStopped {
        let RunAwaitingDose { dosing, active } = self;
        let frontier = RunFrontier::stopped_by_effect(
            dosing.context.manifest().run_coordinate(),
            EffectStage::Dosing(active),
            Some(dosing.last_record),
            dosing.last_dose,
            error,
        );
        RunExecutionStopped::new(dosing.sink, frontier)
    }
}
