//! The mutually-exclusive cause of a run stopping short: a sink write, a server effect, or cleanup.

use anyhow::Error;

use crate::observation::poisoned_tail::PoisonedTail;
use crate::observation::record_id::RecordId;

/// How a run stopped short, as a sum of the three structurally distinct causes — so a
/// [`RunFrontier`](super::run_frontier::RunFrontier) can never encode a contradiction like a sink-write
/// attempt *and* a server-effect error at once. The run coordinate, stage, and last-success markers stay
/// on the frontier because they are shared across all three causes; only the cause-specific evidence
/// lives here, under the variant that owns it. This replaces the earlier flat product of three sibling
/// `Option`s (`attempted_record` / `poison` / `effect_error`) whose exclusion was maintained only by
/// constructor discipline.
///
/// - [`Self::SinkWrite`]: a manifest or observation *sink write* failed. It carries the record whose
///   write was attempted (`None` when the write was refused after the sink was already poisoned, so
///   there was no fresh attempt) and the sink's retained poison (`None` when the failing write did not
///   make the sink terminal). These two are independent *within* a sink-write stop, which is why they
///   are `Option`s here — not because a sink-write stop and some other cause could coexist.
/// - [`Self::Effect`]: a *server effect* — background-seed, initial-set-check, measurement, or
///   post-write check — failed without attempting a sink write, so its only evidence is the stopping
///   `error`, retained verbatim.
/// - [`Self::CleanupStage`]: execution finished cleanly but the obligatory cleanup failed. The failing
///   stage (`Disconnecting`/`Teardown`) is on the frontier and the typed
///   [`RunCleanupFailure`](super::run_cleanup_failure::RunCleanupFailure) is retained separately by the
///   [`RunIncompletion::Cleanup`](super::run_incompletion::RunIncompletion::Cleanup) that holds this
///   frontier, so this variant needs no payload of its own.
///
/// Holds `anyhow::Error` in [`Self::Effect`], so it is `Debug`-only (no `Clone`/`Serialize`) — the
/// effect message is diagnostic evidence, not a serialized output field.
#[derive(Debug)]
pub(crate) enum RunStopEvidence {
    /// A manifest or observation sink write failed: the attempted record (if a fresh attempt) and the
    /// sink's retained poison (if the failing write made it terminal).
    SinkWrite {
        attempted_record: Option<RecordId>,
        poison: Option<PoisonedTail>,
    },
    /// A server effect (background-seed, initial-set-check, measurement, or post-write check) failed.
    Effect { error: Error },
    /// Execution exhausted cleanly but the obligatory cleanup failed at the frontier's stage.
    CleanupStage,
}
