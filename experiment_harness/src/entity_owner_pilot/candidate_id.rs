//! Which deployable candidate an attempt exercises.

use serde::Serialize;

/// The closed set of candidates the spec evaluates.
///
/// Transcribed verbatim from the spec's "Minimal type design" `CandidateId`, including the
/// candidates not yet implemented: the enum is the preregistered inventory, so a later candidate
/// joins the ledger without renumbering or reinterpreting evidence already recorded under this
/// vocabulary. Binding every attempt to one of these values is what stops evidence for two
/// different candidates from being silently pooled — the "category mistake" the spec's minimal type
/// design exists to prevent.
///
/// Only [`Self::EntityOwnerSenderView`] is implemented today (spec WORK LOG, "Start with
/// `EntityOwnerSenderView`"); the rest are declared and unused until their milestones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum CandidateId {
    /// Exact production-composition sender-scoped Chronicle view.
    ChronicleProductionView,
    /// Exact production-composition sender-scoped entity-ownership view — the first implemented
    /// candidate, an analogue of Muninn's real `entity_owner_view`.
    EntityOwnerSenderView,
    /// Exact production-composition sender-scoped message-visibility view.
    MessageVisibilitySenderView,
    /// Exact production-composition sender-scoped control-activity view.
    ControlActivitySenderView,
    /// Control-activity sender view plus the minimal useful sender index.
    IndexedControlActivitySenderView,
    /// Bounded current-state control registry, upserted by activity.
    ControlRegistry,
    /// Authorization-safe exact-key/per-key subscription over a secure entity-owner relation.
    SecureEntityOwnerPerKey,
    /// Raw public-table per-key probe. Retained as an optimizer diagnostic only; the spec forbids
    /// ever recommending it.
    RawPerKeyDiagnostic,
}
