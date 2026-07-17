//! The first post-manifest, pre-dose run state: apply the unmeasured background warm-up slice, once.

use anyhow::Error;

use crate::campaign::effect_stage::EffectStage;
use crate::campaign::run_frontier::RunFrontier;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ManifestWriteReceipt;
use crate::observation::observation_sink::ObservationSink;

use super::RunCheckingInitialSet;
use super::RunExecutionStopped;

/// A run that has written its manifest and now owns the first pre-dose phase: applying the unmeasured
/// pinned background warm-up slice through the normal insert reducers, before any dose. It exposes exactly
/// two transitions — [`Self::stop_background_seed`] on failure and the consuming [`Self::seeded`] on
/// success — and **no** initial-set check: the check is unreachable until the warm-up succeeds, because
/// [`RunCheckingInitialSet`] is minted only by [`Self::seeded`]. Ordering is therefore structural, not a
/// convention: an initial-set check before the warm-up is unrepresentable.
///
/// This phase exists once per run and is consumed by exactly one of its transitions, so a
/// [`RunStage::BackgroundSeed`](crate::campaign::run_stage::RunStage::BackgroundSeed) stop with the
/// warm-up already past — or with any dose landed — cannot be
/// expressed. Owns the sink and manifest receipt by move, handing both to the next phase so the single
/// required output stream and the observations' manifest key are never dropped or duplicated.
pub(crate) struct RunSeedingBackground {
    sink: ObservationSink,
    coord: RunCoordinate,
    manifest: ManifestWriteReceipt,
}

impl RunSeedingBackground {
    /// Enter the warm-up phase immediately after the manifest write's contract returned success, taking
    /// the sink and the manifest receipt. `pub(super)` so only [`RunWritingManifest`'s manifest-write
    /// transition](super::RunWritingManifest) — not a campaign sibling — mints one.
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

    /// Advance to the initial-set check after the warm-up slice's writes returned success. Consumes this
    /// phase, moving the sink and manifest receipt into [`RunCheckingInitialSet`]; past this point the
    /// warm-up stop no longer exists. `pub(in crate::campaign)` so only the run driver advances the run.
    pub(in crate::campaign) fn seeded(self) -> RunCheckingInitialSet {
        RunCheckingInitialSet::new(self.sink, self.coord, self.manifest)
    }

    /// Stop the run because the unmeasured background warm-up seed failed — a pre-dose *server effect*,
    /// after the manifest record and before the first dose. Consumes this phase, keeps the exact
    /// [`RunStage::BackgroundSeed`](crate::campaign::run_stage::RunStage::BackgroundSeed) stage (built from
    /// the typed [`EffectStage`] input) and the original `error` as typed effect evidence, and yields the
    /// same inert [`RunExecutionStopped`] carrier its linear cleanup owner settles: there is no
    /// optional-cleanup or drop-the-cursor escape hatch and no second settlement path.
    /// `last_successful_record` is the manifest record; no dose has landed, so `last_durable_dose` is
    /// `None` by construction of this state.
    pub(in crate::campaign) fn stop_background_seed(self, error: Error) -> RunExecutionStopped {
        let frontier = RunFrontier::stopped_by_effect(
            self.coord,
            EffectStage::BackgroundSeed,
            Some(self.manifest.record()),
            None,
            error,
        );
        RunExecutionStopped::new(self.sink, frontier)
    }
}
