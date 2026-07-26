//! The complete durable identity of one attempt.

use serde::Serialize;

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::CandidateVersion;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;

/// The complete identity of one attempt, per the spec's "Minimal type design" `AttemptKey`.
///
/// Every component is a closed enum, a validated pair, or a structurally-validated index, so an
/// identity naming a nonexistent candidate, scale, stage, or block is unrepresentable. The role
/// reuses the existing [`RunRole`] rather than minting a parallel Arm/Control vocabulary that would
/// drift.
///
/// The scale point is *inside* the identity because every scale point is a separately provisioned
/// fresh server: an attempt measures one scale and one scale only, so a key that did not name it
/// could not distinguish two servers of the same block and role. This is the one shape difference
/// from the completed cumulative Pilot's same-named key, which carried a bare axis because a single
/// attempt walked the whole ladder — and is why that campaign's evidence is not reinterpreted here.
///
/// The full key — including [`RetryOrdinal`] — is the identity a record is written under, so a retry
/// never overwrites its predecessor. The *logical slot* is the same key with the retry disregarded
/// ([`Self::same_logical_slot`]). Both readings come from this one type; there is no separate slot
/// type to keep in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct AttemptKey {
    candidate: CandidateId,
    scale: ScalePoint,
    role: RunRole,
    stage: StageRepetition,
    retry: RetryOrdinal,
    version: CandidateVersion,
}

impl AttemptKey {
    /// Mint an attempt identity from its six components.
    pub(crate) fn new(
        candidate: CandidateId,
        scale: ScalePoint,
        role: RunRole,
        stage: StageRepetition,
        retry: RetryOrdinal,
        version: CandidateVersion,
    ) -> Self {
        Self {
            candidate,
            scale,
            role,
            stage,
            retry,
            version,
        }
    }

    /// Which candidate's measured path this attempt exercised.
    pub(crate) fn candidate(self) -> CandidateId {
        self.candidate
    }

    /// The validated scale this attempt holds fixed for the whole of its measurement.
    pub(crate) fn scale(self) -> ScalePoint {
        self.scale
    }

    /// The implementation version of the candidate path this attempt measured. Read together with
    /// [`Self::candidate`] by
    /// [`SupersededScope`](super::superseded_scope::SupersededScope), whose version scope names
    /// exactly the attempts these two components identify.
    pub(crate) fn version(self) -> CandidateVersion {
        self.version
    }

    /// Whether this attempt measures the module view under test or its matched direct-table
    /// control. The driver's sole branch: it selects the subscription target, the read-back, and
    /// the expected result set together, so those three cannot disagree about which side is running.
    pub(crate) fn role(self) -> RunRole {
        self.role
    }

    /// Which retry of its logical slot this attempt is. Read by analysis, which selects the valid
    /// complete attempt with the lowest ordinal.
    pub(crate) fn retry(self) -> RetryOrdinal {
        self.retry
    }

    /// Whether two identities address the same logical slot — every component equal except the
    /// retry ordinal.
    pub(crate) fn same_logical_slot(self, other: Self) -> bool {
        self.candidate == other.candidate
            && self.scale == other.scale
            && self.role == other.role
            && self.stage == other.stage
            && self.version == other.version
    }

    /// Whether two identities belong to the same ladder — the same `(candidate, block, role, axis,
    /// version)`, differing only in which rung they sit at and which retry they are.
    ///
    /// This is the grouping the endpoint factor is computed over: `T = S_last / S_first` compares two
    /// rungs of one ladder, so evidence from a different block, role, candidate, axis, or version
    /// must never be pooled into it.
    pub(crate) fn same_ladder(self, other: Self) -> bool {
        self.candidate == other.candidate
            && self.scale.axis() == other.scale.axis()
            && self.role == other.role
            && self.stage == other.stage
            && self.version == other.version
    }
}
