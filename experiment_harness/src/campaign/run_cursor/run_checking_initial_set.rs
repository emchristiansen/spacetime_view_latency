//! The second post-manifest, pre-dose run state: check the initial expected set, once, then dose.

use anyhow::Error;

use crate::campaign::effect_stage::EffectStage;
use crate::campaign::run_frontier::RunFrontier;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ManifestWriteReceipt;
use crate::observation::observation_sink::ObservationSink;

use super::RunDosing;
use super::RunExecutionStopped;

/// A run whose warm-up slice has landed and now owns the second pre-dose phase: asserting the pre-dose
/// subscribed result set equals the seed-derived expected set, before the first dose. Reachable **only**
/// from [`RunSeedingBackground::seeded`](super::RunSeedingBackground), so it cannot precede the warm-up. It
/// exposes exactly two transitions — [`Self::stop_initial_set_check`] on failure and the consuming
/// [`Self::checked`] into [`RunDosing`] on success — and **no** warm-up stop, because that phase has been
/// consumed. A [`RunStage::BackgroundSeed`](crate::campaign::run_stage::RunStage::BackgroundSeed) failure
/// after the initial-set check is therefore
/// unrepresentable, exactly as an initial check before the warm-up is.
///
/// This phase exists once per run and is consumed by exactly one of its transitions. Owns the sink and
/// manifest receipt by move, handing both to the initial [`RunDosing`] on success so the single required
/// output stream and the observations' manifest key are never dropped or duplicated.
pub(crate) struct RunCheckingInitialSet {
    sink: ObservationSink,
    coord: RunCoordinate,
    manifest: ManifestWriteReceipt,
}

impl RunCheckingInitialSet {
    /// Enter the initial-set-check phase after the warm-up slice returned success, taking the sink and the
    /// manifest receipt. `pub(super)` so only [`RunSeedingBackground::seeded`](super::RunSeedingBackground)
    /// — not a campaign sibling — mints one, keeping the seed→check order structural.
    pub(super) fn new(
        sink: ObservationSink,
        coord: RunCoordinate,
        manifest: ManifestWriteReceipt,
    ) -> Self {
        Self {
            sink,
            coord,
            manifest,
        }
    }

    /// Begin the dose ladder after the initial-set check returned success. Consumes this phase, moving the
    /// sink and manifest receipt into the initial [`RunDosing`]; past this point neither pre-dose stop
    /// exists. `pub(in crate::campaign)` so only the run driver advances the run.
    pub(in crate::campaign) fn checked(self) -> RunDosing {
        RunDosing::begin(self.sink, self.coord, self.manifest)
    }

    /// Stop the run because the pre-dose initial-set check failed — a pre-dose *server effect*, after the
    /// warm-up slice and before the first dose. Consumes this phase, keeps the exact
    /// [`RunStage::InitialSetCheck`](crate::campaign::run_stage::RunStage::InitialSetCheck) stage (built from
    /// the typed [`EffectStage`] input) and the original `error` as typed effect evidence, and yields the
    /// same inert [`RunExecutionStopped`] carrier its linear cleanup owner settles: there is no
    /// optional-cleanup or drop-the-cursor escape hatch and no second settlement path.
    /// `last_successful_record` is the manifest record; no dose has landed, so `last_durable_dose` is
    /// `None` by construction of this state.
    pub(in crate::campaign) fn stop_initial_set_check(self, error: Error) -> RunExecutionStopped {
        let frontier = RunFrontier::stopped_by_effect(
            self.coord,
            EffectStage::InitialSetCheck,
            Some(self.manifest.record()),
            None,
            error,
        );
        RunExecutionStopped::new(self.sink, frontier)
    }
}
