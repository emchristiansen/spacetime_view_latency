//! The complete durable identity of one attempt.

use serde::Serialize;

use crate::entity_owner_pilot::candidate_id::CandidateId;
use crate::entity_owner_pilot::candidate_version::CandidateVersion;
use crate::entity_owner_pilot::experiment_axis::ExperimentAxis;
use crate::entity_owner_pilot::retry_ordinal::RetryOrdinal;
use crate::entity_owner_pilot::stage_repetition::StageRepetition;
use crate::plan::run_role::RunRole;

/// The complete identity of one attempt, as transcribed from the spec's "Minimal type design"
/// `AttemptKey`.
///
/// Every component is a closed enum or a structurally-validated index, so an attempt identity that
/// names a nonexistent candidate, axis, stage, block, or ladder position is unrepresentable rather
/// than merely unlikely. The role reuses the existing
/// [`RunRole`](crate::plan::run_role::RunRole) rather than minting a parallel Arm/Control vocabulary
/// — the spec directs reusing existing harness types where they already enforce the invariant, and a
/// second Arm/Control enum is precisely the kind of duplicated contract that drifts.
///
/// **Identity versus logical slot.** The full key — including [`RetryOrdinal`] — is the durable
/// identity a ledger record is written under, so a retry never overwrites its predecessor. The
/// *logical slot* is the same key with the retry ordinal disregarded, which is the granularity at
/// which the spec says analysis "selects exactly one current valid complete attempt". Both readings
/// come from this one type; there is no separate slot type to keep in sync.
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

    /// The candidate this attempt exercises.
    pub(crate) fn candidate(self) -> CandidateId {
        self.candidate
    }

    /// The frozen finite range this attempt walks.
    pub(crate) fn axis(self) -> ExperimentAxis {
        self.axis
    }

    /// Whether this attempt is the arm under test or its matched control.
    pub(crate) fn role(self) -> RunRole {
        self.role
    }

    /// The stage and, where the stage repeats, the block index within it.
    pub(crate) fn stage(self) -> StageRepetition {
        self.stage
    }

    /// Which retry of this attempt's logical slot it is.
    pub(crate) fn retry(self) -> RetryOrdinal {
        self.retry
    }

    /// The candidate implementation version this attempt measured.
    pub(crate) fn version(self) -> CandidateVersion {
        self.version
    }

    /// Whether two attempt identities address the same logical slot — every component equal except
    /// the retry ordinal. This is the equivalence the spec's "exactly one current valid complete
    /// attempt per logical slot" is stated over.
    pub(crate) fn same_logical_slot(self, other: Self) -> bool {
        self.candidate == other.candidate
            && self.axis == other.axis
            && self.role == other.role
            && self.stage == other.stage
            && self.version == other.version
    }
}
