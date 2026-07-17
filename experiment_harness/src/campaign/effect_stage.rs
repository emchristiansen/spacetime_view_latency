//! The lifecycle stages at which a run's *server effect* can fail: warm-up, initial check, or a dose.

use crate::dataset::dose_index::DoseIndex;

use super::run_stage::RunStage;

/// The exact subset of [`RunStage`] a *server effect* can fail in — the pre-dose background warm-up, the
/// pre-dose initial-set check, or a dose's measured writes/post-write check.
/// [`RunFrontier::stopped_by_effect`] takes this rather than an arbitrary [`RunStage`] and widens it
/// internally, so an effect frontier tagged with the manifest sink-write stage (`WritingManifest`) or a
/// cleanup stage (`Disconnecting`/`Teardown`) is unrepresentable.
///
/// [`RunFrontier::stopped_by_effect`]: super::run_frontier::RunFrontier::stopped_by_effect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectStage {
    /// Applying the unmeasured pinned background warm-up slice, before any dose.
    BackgroundSeed,
    /// Asserting the pre-dose subscribed result set equals the seed-derived expected set.
    InitialSetCheck,
    /// A dose's measured writes and post-write correctness/event checks, at the given ladder index.
    Dosing(DoseIndex),
}

impl EffectStage {
    /// Widen to the corresponding [`RunStage`]. `pub(in crate::campaign)` so only a frontier constructor
    /// widens it.
    pub(in crate::campaign) fn stage(self) -> RunStage {
        match self {
            EffectStage::BackgroundSeed => RunStage::BackgroundSeed,
            EffectStage::InitialSetCheck => RunStage::InitialSetCheck,
            EffectStage::Dosing(dose) => RunStage::Dosing(dose),
        }
    }
}
