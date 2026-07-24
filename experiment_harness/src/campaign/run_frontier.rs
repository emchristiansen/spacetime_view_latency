//! The structural record of where and how a single run stopped short of completion.

use anyhow::Error;

use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::poisoned_tail::PoisonedTail;
use crate::observation::record_id::RecordId;

use super::effect_stage::EffectStage;
use super::run_cleanup_failure::RunCleanupFailure;
use super::run_stage::RunStage;
use super::run_stop_evidence::RunStopEvidence;
use super::sink_write_stage::SinkWriteStage;

/// Everything needed to resume or diagnose a run that did not complete: which run, the stage it
/// stopped in, the last record and dose whose writer contract returned success, and the typed
/// [`RunStopEvidence`] naming *how* it stopped. Retaining a record/dose identity is evidence the writer
/// contract returned success for it, not a claim of physical crash persistence.
///
/// `last_successful_record` is `Option` (none before the manifest write) and is kept separate from
/// `last_durable_dose`, which is `None` until at least the first dose write's contract returns success.
/// Both are shared across every way a run can stop, so they live here.
///
/// The cause-specific evidence — a sink write's attempted record and poison, a server effect's error,
/// or a cleanup stage carrying neither — lives under the [`RunStopEvidence`] variants, not as sibling
/// `Option`s here. That makes a contradictory frontier (a sink-write attempt *and* an effect error at
/// once, say) unrepresentable, rather than merely unbuilt-by-discipline: there is one field for the
/// cause, and it is exactly one variant. Like its
/// [`RunCleanupFailure`](super::run_cleanup_failure::RunCleanupFailure) sibling the effect variant holds
/// `anyhow::Error`, so `RunFrontier` stays `Debug`-only (no `Clone`/`Serialize`); the effect message is
/// diagnostic evidence, not a serialized output field.
#[derive(Debug)]
pub(crate) struct RunFrontier {
    run: RunCoordinate,
    stage: RunStage,
    last_successful_record: Option<RecordId>,
    last_durable_dose: Option<DoseIndex>,
    cause: RunStopEvidence,
}

impl RunFrontier {
    /// Record a run frontier from a *sink write's* failure at the moment it stopped — the manifest or
    /// observation write whose contract did not return success. The stage is a [`SinkWriteStage`] (widened
    /// to [`RunStage`] internally), so a sink-write frontier can only be tagged `WritingManifest` or
    /// `Dosing`, never a pre-dose effect or cleanup stage. The cause is [`RunStopEvidence::SinkWrite`],
    /// carrying the record whose write was attempted (if any) and the sink's retained poison (if it became
    /// terminal). `pub(in crate::campaign)` so only a run cursor's transition builds one.
    pub(in crate::campaign) fn stopped_by_sink_write(
        run: RunCoordinate,
        sink_stage: SinkWriteStage,
        last_successful_record: Option<RecordId>,
        last_durable_dose: Option<DoseIndex>,
        attempted_record: Option<RecordId>,
        poison: Option<PoisonedTail>,
    ) -> Self {
        Self {
            run,
            stage: sink_stage.stage(),
            last_successful_record,
            last_durable_dose,
            cause: RunStopEvidence::SinkWrite {
                attempted_record,
                poison,
            },
        }
    }

    /// Record a run frontier from a *server effect's* failure — a background-seed, initial-set-check,
    /// measurement, or post-write check step that stopped the run without attempting a sink write. The
    /// stage is an [`EffectStage`] (widened to [`RunStage`] internally), so an effect frontier can only be
    /// tagged `BackgroundSeed`, `InitialSetCheck`, or `Dosing`, never the manifest sink-write stage or a
    /// cleanup stage. The cause is [`RunStopEvidence::Effect`] retaining the stopping `error` verbatim, so
    /// the incompletion names exactly why execution stopped. `pub(in crate::campaign)` so only a run
    /// cursor's effect-failure transition builds one.
    pub(in crate::campaign) fn stopped_by_effect(
        run: RunCoordinate,
        effect_stage: EffectStage,
        last_successful_record: Option<RecordId>,
        last_durable_dose: Option<DoseIndex>,
        error: Error,
    ) -> Self {
        Self {
            run,
            stage: effect_stage.stage(),
            last_successful_record,
            last_durable_dose,
            cause: RunStopEvidence::Effect { error },
        }
    }

    /// Record a cleanup-stage frontier: execution exhausted cleanly, but the obligatory cleanup failed.
    /// The stage is **derived** from the `failure` ([`RunCleanupFailure::stage`], `Disconnecting` or
    /// `Teardown`), not supplied — so a cleanup-stage frontier at a pre-cleanup stage like
    /// `WritingManifest`/`Dosing` cannot be constructed. The cause is [`RunStopEvidence::CleanupStage`];
    /// the typed [`RunCleanupFailure`] itself is retained by the
    /// [`RunIncompletion::Cleanup`](super::run_incompletion::RunIncompletion::Cleanup) that holds this
    /// frontier (borrowed here, moved there), so there is no sink attempt, poison, or effect error to
    /// carry. `pub(in crate::campaign)` so only the linear cleanup owner's settlement builds one.
    pub(in crate::campaign) fn cleanup_stage(
        run: RunCoordinate,
        failure: &RunCleanupFailure,
        last_successful_record: Option<RecordId>,
        last_durable_dose: Option<DoseIndex>,
    ) -> Self {
        Self {
            run,
            stage: failure.stage(),
            last_successful_record,
            last_durable_dose,
            cause: RunStopEvidence::CleanupStage,
        }
    }

    /// The run that stopped short.
    pub(crate) fn run(&self) -> &RunCoordinate {
        &self.run
    }

    /// The stage the run stopped in.
    pub(crate) fn stage(&self) -> RunStage {
        self.stage
    }

    /// The last record whose writer contract returned success, if any.
    pub(crate) fn last_successful_record(&self) -> Option<RecordId> {
        self.last_successful_record
    }

    /// The last dose whose observation write's writer contract returned success, if any.
    pub(crate) fn last_durable_dose(&self) -> Option<DoseIndex> {
        self.last_durable_dose
    }

    /// The typed cause of the stop — a sink write, a server effect, or a cleanup stage.
    pub(crate) fn cause(&self) -> &RunStopEvidence {
        &self.cause
    }

    /// The record whose write was attempted when the run stopped, if it stopped at a sink write with a
    /// fresh attempt. `None` for an effect or cleanup-stage stop, which attempt no sink write.
    pub(crate) fn attempted_record(&self) -> Option<RecordId> {
        match &self.cause {
            RunStopEvidence::SinkWrite {
                attempted_record, ..
            } => *attempted_record,
            RunStopEvidence::Effect { .. } | RunStopEvidence::CleanupStage => None,
        }
    }

    /// The sink's retained poison reason, if a sink-write stop made it terminal. `None` for an effect or
    /// cleanup-stage stop.
    pub(crate) fn poison(&self) -> Option<&PoisonedTail> {
        match &self.cause {
            RunStopEvidence::SinkWrite { poison, .. } => poison.as_ref(),
            RunStopEvidence::Effect { .. } | RunStopEvidence::CleanupStage => None,
        }
    }

    /// The server effect's failure that stopped the run, if it stopped in an effect stage
    /// (`BackgroundSeed`, `InitialSetCheck`, or a `Dosing` measurement/post-write check) rather than at a
    /// sink write or in cleanup.
    pub(crate) fn effect_error(&self) -> Option<&Error> {
        match &self.cause {
            RunStopEvidence::Effect { error } => Some(error),
            RunStopEvidence::SinkWrite { .. } | RunStopEvidence::CleanupStage => None,
        }
    }
}
