//! The complete durable identity of one attempt.

use serde::Serialize;

use crate::control_registry_discovery_screen::candidate_id::CandidateId;
use crate::control_registry_discovery_screen::candidate_version::CandidateVersion;
use crate::control_registry_discovery_screen::experiment_axis::ExperimentAxis;
use crate::control_registry_discovery_screen::retry_ordinal::RetryOrdinal;
use crate::control_registry_discovery_screen::screen_block_index::ScreenBlockIndex;
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::control_registry_discovery_screen::screen_target::ScreenTarget;
use crate::control_registry_discovery_screen::stage_repetition::StageRepetition;
use crate::plan::run_role::RunRole;

/// The complete identity of one attempt, per the spec's "Minimal type design": candidate, scale
/// point, role, retry ordinal, and candidate version, so stale or mismatched results cannot join.
///
/// Seven components rather than the Pilot's six, and the extra one is [`ScreenRung`]. The Pilot
/// walks the whole ladder progressively *inside* one attempt, so its identity needs no rung; this
/// screen provisions a separate instance per endpoint, so the rung is part of what an attempt *is*.
/// Omitting it would make a block's low and high attempts collide as one logical slot.
///
/// The role reuses the existing [`RunRole`] rather than minting a parallel Arm/Control vocabulary
/// that would drift. Together with the candidate it recovers the measured target exactly — see
/// [`Self::target`] — so the identity carries no separate target field able to contradict it.
///
/// The full key, including [`RetryOrdinal`], is the identity a record is written under, so a retry
/// never overwrites its predecessor. The *logical slot* is the same key with the retry disregarded
/// ([`Self::same_logical_slot`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct AttemptKey {
    candidate: CandidateId,
    axis: ExperimentAxis,
    rung: ScreenRung,
    role: RunRole,
    stage: StageRepetition,
    retry: RetryOrdinal,
    version: CandidateVersion,
}

impl AttemptKey {
    /// Mint an attempt identity from its seven components.
    pub(crate) fn new(
        candidate: CandidateId,
        axis: ExperimentAxis,
        rung: ScreenRung,
        role: RunRole,
        stage: StageRepetition,
        retry: RetryOrdinal,
        version: CandidateVersion,
    ) -> Self {
        Self {
            candidate,
            axis,
            rung,
            role,
            stage,
            retry,
            version,
        }
    }

    /// The endpoint this attempt measures.
    pub(crate) fn rung(self) -> ScreenRung {
        self.rung
    }

    /// The candidate this attempt produces evidence for.
    pub(crate) fn candidate(self) -> CandidateId {
        self.candidate
    }

    /// Whether this attempt measures a candidate arm or its matched composition Control.
    pub(crate) fn role(self) -> RunRole {
        self.role
    }

    /// Which attempt at this logical slot the identity names.
    pub(crate) fn retry(self) -> RetryOrdinal {
        self.retry
    }

    /// The block this attempt belongs to.
    pub(crate) fn stage_block(self) -> ScreenBlockIndex {
        match self.stage {
            StageRepetition::Screen(block) => block,
        }
    }

    /// The measured target this identity denotes.
    ///
    /// The driver's sole branch: it selects the timed subscription, the expected cardinality, and
    /// the ledger tag together from one value, so those three cannot disagree about which side is
    /// running.
    pub(crate) fn target(self) -> ScreenTarget {
        ScreenTarget::of(self.candidate, self.role)
    }

    /// Whether two identities address the same logical slot — every component equal except the retry
    /// ordinal.
    pub(crate) fn same_logical_slot(self, other: Self) -> bool {
        self.candidate == other.candidate
            && self.axis == other.axis
            && self.rung == other.rung
            && self.role == other.role
            && self.stage == other.stage
            && self.version == other.version
    }
}
