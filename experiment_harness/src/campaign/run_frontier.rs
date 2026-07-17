//! The structural record of where and how a single run stopped short of completion.

use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::poisoned_tail::PoisonedTail;
use crate::observation::record_id::RecordId;

use super::run_stage::RunStage;

/// Everything needed to resume or diagnose a run that did not complete: which run, the stage it
/// stopped in, the last record and dose whose writer contract returned success, the record whose write
/// was attempted when it stopped (if any), and the sink's retained poison reason (if the sink became
/// terminal). Retaining a record/dose identity is evidence the writer contract returned success for it,
/// not a claim of physical crash persistence.
///
/// `attempted_record` is `Option` because several stops have no fresh in-flight record: a pre-write
/// (`WritingManifest`) stop before the first attempt, a disconnect/teardown stop after the ladder, and
/// a write refused after the sink was already poisoned all have no attempt of their own.
/// `last_successful_record` is likewise `Option` (none before the manifest write) and is kept separate
/// from `last_durable_dose`, which is `None` until at least the first dose write's contract returns
/// success.
#[derive(Debug)]
pub(crate) struct RunFrontier {
    run: RunCoordinate,
    stage: RunStage,
    last_successful_record: Option<RecordId>,
    last_durable_dose: Option<DoseIndex>,
    attempted_record: Option<RecordId>,
    poison: Option<PoisonedTail>,
}

impl RunFrontier {
    /// Record a run frontier from the run's live state at the moment it stopped. `pub(in
    /// crate::campaign)` so only a run cursor's transition builds one.
    pub(in crate::campaign) fn new(
        run: RunCoordinate,
        stage: RunStage,
        last_successful_record: Option<RecordId>,
        last_durable_dose: Option<DoseIndex>,
        attempted_record: Option<RecordId>,
        poison: Option<PoisonedTail>,
    ) -> Self {
        Self {
            run,
            stage,
            last_successful_record,
            last_durable_dose,
            attempted_record,
            poison,
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

    /// The record whose write was attempted when the run stopped, if there was a fresh attempt.
    pub(crate) fn attempted_record(&self) -> Option<RecordId> {
        self.attempted_record
    }

    /// The sink's retained poison reason, if a persist failure made it terminal.
    pub(crate) fn poison(&self) -> Option<&PoisonedTail> {
        self.poison.as_ref()
    }
}
