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

    /// The identity of this logical slot's one permitted retry, or `None` when this identity is
    /// already that retry.
    ///
    /// **Why the transition lives on the identity and not at the driver that wants it.** Two
    /// reasons, neither of which is coordinate transposition — [`Self::new`] takes six distinct
    /// types, so a swapped pair does not compile, and a seventh component would change its arity and
    /// break every call site rather than pass silently.
    ///
    /// The first is access: [`StageRepetition`] is the one coordinate this type deliberately does
    /// not expose, and rebuilding a key elsewhere would require publishing it for no reason but
    /// this. The second is that reconstruction names *values*, and a call site can supply a
    /// well-typed wrong one — the ladder's other rung, the matched role, another block's
    /// repetition — producing a valid identity for an attempt that was never run. Copying `self` and
    /// advancing one closed field names no coordinate at all, so there is none to get wrong. It is
    /// the constructive direction of [`Self::same_logical_slot`], which is why the two live together
    /// and cannot drift apart.
    ///
    /// **Why this rather than a setter.** An arbitrary-ordinal setter could also move a retry *back*
    /// to [`RetryOrdinal::ORIGINAL`], minting an identity claiming to be the attempt it was derived
    /// from — which would overwrite its predecessor's evidence, the one thing the retry ordinal
    /// exists to prevent. This takes no parameter, so the only reachable transition is the permitted
    /// one.
    ///
    /// **The cap is the ordinal type's, not a rule restated here.** [`RetryOrdinal`] holds exactly
    /// two values, so an identity that is not the original is the retry and has no successor. The
    /// test asks "is this the original?" rather than "is this the retry?" deliberately: were a third
    /// ordinal ever added inside that type's sealed module, this yields no retry rather than
    /// silently granting one.
    ///
    /// It promises nothing about *scheduling*. Whether a retry may be run at all is the outcome
    /// question answered by
    /// [`retry_eligibility`](super::terminal_attempt_record::TerminalAttemptRecord::retry_eligibility),
    /// and when it runs is the driver's.
    pub(crate) fn next_retry(self) -> Option<Self> {
        if self.retry != RetryOrdinal::ORIGINAL {
            return None;
        }
        Some(Self {
            retry: RetryOrdinal::RETRY,
            ..self
        })
    }

    /// The complete identity as one frozen canonical string — the spelling its retained artifacts
    /// are filed under.
    ///
    /// **Every coordinate, because every coordinate distinguishes an attempt.** Two attempts can
    /// agree on all but one component, so a tag omitting any of the six would collide two distinct
    /// identities' evidence: the block coordinate in particular is the *only* thing separating the
    /// matched blocks' same-rung, same-role attempts.
    ///
    /// **Nothing here is `Debug`, a variant name, or serde output.** Each token and number comes
    /// from an explicit canonical accessor on its own typed component, following
    /// [`Cell::canonical_tag`](crate::plan::cell::Cell::canonical_tag) and the keyed-sort subject
    /// in [`Schedule`](crate::plan::schedule::Schedule) — including its semicolon-separated
    /// `key=value` shape. That is what stops a Rust rename or a serde attribute from moving
    /// evidence a reader has already been pointed at.
    ///
    /// The result is one safe filename component: the frozen vocabulary contains no path separator,
    /// no `.` or `..` spelling, and nothing empty, and it stays far below the POSIX component
    /// limit.
    pub(crate) fn canonical_tag(self) -> String {
        format!(
            "candidate={};axis={};rung={};role={};stage={};block={};version={};retry={}",
            self.candidate.canonical_tag(),
            self.scale.axis().canonical_tag(),
            self.scale.rung().get(),
            self.role.canonical_tag(),
            self.stage.stage_tag(),
            self.stage.block_index(),
            self.version.get(),
            self.retry.get(),
        )
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

#[cfg(test)]
mod tests;
