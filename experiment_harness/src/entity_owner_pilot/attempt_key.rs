//! The complete durable identity of one attempt.

use serde::Serialize;

use crate::entity_owner_pilot::candidate_id::CandidateId;
use crate::entity_owner_pilot::candidate_version::CandidateVersion;
use crate::entity_owner_pilot::experiment_axis::ExperimentAxis;
use crate::entity_owner_pilot::retry_ordinal::RetryOrdinal;
use crate::entity_owner_pilot::stage_repetition::StageRepetition;
use crate::plan::run_role::RunRole;

/// The complete identity of one attempt, per the spec's "Minimal type design" `AttemptKey`.
///
/// Every component is a closed enum or a structurally-validated index, so an identity naming a
/// nonexistent candidate, axis, stage, or block is unrepresentable. The role reuses the existing
/// [`RunRole`] rather than minting a parallel Arm/Control vocabulary that would drift.
///
/// The full key — including [`RetryOrdinal`] — is the identity a record is written under, so a retry
/// never overwrites its predecessor. The *logical slot* is the same key with the retry disregarded
/// ([`Self::same_logical_slot`]). Both readings come from this one type; there is no separate slot
/// type to keep in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct AttemptKey {
    candidate: CandidateId,
    axis: ExperimentAxis,
    role: RunRole,
    stage: StageRepetition,
    retry: RetryOrdinal,
    version: CandidateVersion,
}

impl AttemptKey {
    /// Mint an attempt identity from its six components.
    pub(crate) fn new(
        candidate: CandidateId,
        axis: ExperimentAxis,
        role: RunRole,
        stage: StageRepetition,
        retry: RetryOrdinal,
        version: CandidateVersion,
    ) -> Self {
        Self {
            candidate,
            axis,
            role,
            stage,
            retry,
            version,
        }
    }

    /// Whether this attempt measures the module view under test or its matched direct-table
    /// control. The driver's sole branch: it selects the subscription target, the read-back, and
    /// the expected result set together, so those three cannot disagree about which side is running.
    pub(crate) fn role(self) -> RunRole {
        self.role
    }

    /// Whether two identities address the same logical slot — every component equal except the
    /// retry ordinal.
    pub(crate) fn same_logical_slot(self, other: Self) -> bool {
        self.candidate == other.candidate
            && self.axis == other.axis
            && self.role == other.role
            && self.stage == other.stage
            && self.version == other.version
    }
}
