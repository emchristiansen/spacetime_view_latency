//! The first post-manifest, pre-dose run state: apply the unmeasured background warm-up slice, once.

use anyhow::Error;

use crate::campaign::effect_stage::EffectStage;
use crate::campaign::run_cursor::RunExecutionStopped;
use crate::campaign::run_frontier::RunFrontier;
use crate::dataset::run_dataset::RunDataset;

use crate::campaign::run_cursor::run_writing_manifest::written_run::WrittenRun;

mod run_checking_initial_set;

pub(crate) use run_checking_initial_set::RunAwaitingDose;
pub(crate) use run_checking_initial_set::RunDosing;

use run_checking_initial_set::RunCheckingInitialSet;

/// A run that has written its manifest and now owns the first pre-dose phase: applying the unmeasured
/// pinned background warm-up slice through the normal insert reducers, before any dose. It exposes exactly
/// two transitions — [`Self::stop_background_seed`] on failure and the consuming [`Self::seeded`] on
/// success — and **no** initial-set check: the check is unreachable until the warm-up succeeds, because
/// [`RunCheckingInitialSet`] is minted only by [`Self::seeded`]. Ordering is therefore structural, not a
/// convention: an initial-set check before the warm-up is unrepresentable.
///
/// As a child module of [`RunWritingManifest`](super::RunWritingManifest), this state's constructor
/// [`Self::new`] is `pub(super)`, confined to that predecessor's subtree, so no module *outside* the nested
/// cursor tree can mint one to bypass the manifest write (within the sealed subtree the visibility also
/// reaches descendants — see [`WrittenRun`]). It owns the single [`WrittenRun`] — the sink, bound context,
/// and manifest receipt as one value — so no externally reachable API pairs a loose sink with a foreign
/// context. This phase exists once per run and is consumed by exactly one of its transitions, so a
/// [`RunStage::BackgroundSeed`](crate::campaign::run_stage::RunStage::BackgroundSeed) stop with the warm-up
/// already past — or with any dose landed — cannot be expressed.
pub(crate) struct RunSeedingBackground {
    written: WrittenRun,
}

impl RunSeedingBackground {
    /// Enter the warm-up phase immediately after the manifest write's contract returned success, taking the
    /// [`WrittenRun`] the manifest-write transition minted (the sink, bound context, and receipt as one
    /// value). The sink, run coordinate, and manifest record are *not* separate fields: they are read from
    /// the single `written` value wherever needed, so none can drift from the manifest they belong to.
    /// `pub(super)`, confining construction to the [`RunWritingManifest`](super::RunWritingManifest) subtree;
    /// the implemented call site is its manifest-write transition, and the `written` it passes can only have
    /// come from the real write (its sole constructor). Rust `pub(super)` also reaches the parent's
    /// descendants, so this is the sealed-subtree boundary rather than single-caller enforcement.
    pub(super) fn new(written: WrittenRun) -> Self {
        Self { written }
    }

    /// The bound run context (manifest + resolved dataset), borrowed so the run driver can derive this
    /// run's background warm-up slice from the run's own dataset — never from a parallel value.
    /// `pub(in crate::campaign)` so only the run driver reads it.
    pub(in crate::campaign) fn run_dataset(&self) -> &RunDataset {
        self.written.context()
    }

    /// Advance to the initial-set check after the warm-up slice's writes returned success. Consumes this
    /// phase, moving the bound [`WrittenRun`] into [`RunCheckingInitialSet`] — the single owned value threads
    /// on. Past this point the warm-up stop no longer exists. `pub(in crate::campaign)` so only the run
    /// driver advances the run.
    pub(in crate::campaign) fn seeded(self) -> RunCheckingInitialSet {
        RunCheckingInitialSet::new(self.written)
    }

    /// Stop the run because the unmeasured background warm-up seed failed — a pre-dose *server effect*,
    /// after the manifest record and before the first dose. Consumes this phase, keeps the exact
    /// [`RunStage::BackgroundSeed`](crate::campaign::run_stage::RunStage::BackgroundSeed) stage (built from
    /// the typed [`EffectStage`] input) and the original `error` as typed effect evidence, and yields the
    /// same inert [`RunExecutionStopped`] carrier its linear cleanup owner settles: there is no
    /// optional-cleanup or drop-the-cursor escape hatch and no second settlement path.
    /// `last_successful_record` is the manifest record read from the bundle; no dose has landed, so
    /// `last_durable_dose` is `None` by construction of this state. The recovered sink is drawn from the same
    /// bundle, so the stopped run and its output stream cannot disagree.
    pub(in crate::campaign) fn stop_background_seed(self, error: Error) -> RunExecutionStopped {
        let frontier = RunFrontier::stopped_by_effect(
            self.written.context().manifest().run_coordinate(),
            EffectStage::BackgroundSeed,
            Some(self.written.receipt().record()),
            None,
            error,
        );
        RunExecutionStopped::new(self.written.into_parts().0, frontier)
    }
}
