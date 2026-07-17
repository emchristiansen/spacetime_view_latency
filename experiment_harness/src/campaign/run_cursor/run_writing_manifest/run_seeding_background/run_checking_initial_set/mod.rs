//! The second post-manifest, pre-dose run state: check the initial expected set, once, then dose.

use anyhow::Error;

use crate::campaign::effect_stage::EffectStage;
use crate::campaign::run_cursor::RunExecutionStopped;
use crate::campaign::run_frontier::RunFrontier;
use crate::dataset::run_dataset::RunDataset;

use crate::campaign::run_cursor::run_writing_manifest::written_run::WrittenRun;

mod run_dosing;

pub(crate) use run_dosing::RunAwaitingDose;
pub(crate) use run_dosing::RunDosing;

/// A run whose warm-up slice has landed and now owns the second pre-dose phase: asserting the pre-dose
/// subscribed result set equals the seed-derived expected set, before the first dose. Reachable **only**
/// from [`RunSeedingBackground::seeded`](super::RunSeedingBackground), so it cannot precede the warm-up. It
/// exposes exactly two transitions — [`Self::stop_initial_set_check`] on failure and the consuming
/// [`Self::checked`] into [`RunDosing`] on success — and **no** warm-up stop, because that phase has been
/// consumed. A [`RunStage::BackgroundSeed`](crate::campaign::run_stage::RunStage::BackgroundSeed) failure
/// after the initial-set check is therefore unrepresentable, exactly as an initial check before the warm-up
/// is.
///
/// As a child module of [`RunSeedingBackground`](super::RunSeedingBackground), this state's constructor
/// [`Self::new`] is `pub(super)`, confined to that predecessor's subtree, so no module *outside* the nested
/// cursor tree can mint one to bypass the warm-up. This phase exists once per run and is consumed by exactly
/// one of its transitions. It owns the single [`WrittenRun`] (the sink, bound context, and manifest receipt
/// as one value), handing it to the initial [`RunDosing`] on success so the output stream and the run's
/// durable-progress evidence are never dropped, duplicated, or paired anew.
pub(crate) struct RunCheckingInitialSet {
    written: WrittenRun,
}

impl RunCheckingInitialSet {
    /// Enter the initial-set-check phase after the warm-up slice returned success, taking the bound
    /// [`WrittenRun`]. The sink, run coordinate, and manifest record are read from that single value, never
    /// separate fields. `pub(super)`, confining construction to the
    /// [`RunSeedingBackground`](super::RunSeedingBackground) subtree; the implemented call site is its
    /// `seeded` transition, keeping the seed→check order structural. The seal is subtree-level — `pub(super)`
    /// also reaches descendants — not single-caller enforcement.
    pub(super) fn new(written: WrittenRun) -> Self {
        Self { written }
    }

    /// The bound run context (manifest + resolved dataset), borrowed so the run driver can subscribe to
    /// this run's own target and check the pre-dose set against this run's own dataset — never a parallel
    /// value. `pub(in crate::campaign)` so only the run driver reads it.
    pub(in crate::campaign) fn run_dataset(&self) -> &RunDataset {
        self.written.context()
    }

    /// Begin the dose ladder after the initial-set check returned success. Consumes this phase, moving the
    /// bound [`WrittenRun`] into the initial [`RunDosing`]; past this point neither pre-dose stop exists.
    /// `pub(in crate::campaign)` so only the run driver advances the run.
    pub(in crate::campaign) fn checked(self) -> RunDosing {
        RunDosing::begin(self.written)
    }

    /// Stop the run because the pre-dose initial-set check failed — a pre-dose *server effect*, after the
    /// warm-up slice and before the first dose. Consumes this phase, keeps the exact
    /// [`RunStage::InitialSetCheck`](crate::campaign::run_stage::RunStage::InitialSetCheck) stage (built from
    /// the typed [`EffectStage`] input) and the original `error` as typed effect evidence, and yields the
    /// same inert [`RunExecutionStopped`] carrier its linear cleanup owner settles: there is no
    /// optional-cleanup or drop-the-cursor escape hatch and no second settlement path.
    /// `last_successful_record` is the manifest record read from the bundle; no dose has landed, so
    /// `last_durable_dose` is `None` by construction of this state. The recovered sink is drawn from the same
    /// bundle, so the stopped run and its output stream cannot disagree.
    pub(in crate::campaign) fn stop_initial_set_check(self, error: Error) -> RunExecutionStopped {
        let frontier = RunFrontier::stopped_by_effect(
            self.written.context().manifest().run_coordinate(),
            EffectStage::InitialSetCheck,
            Some(self.written.receipt().record()),
            None,
            error,
        );
        RunExecutionStopped::new(self.written.into_parts().0, frontier)
    }
}
