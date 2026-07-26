//! What a method-supersession record invalidates.

use serde::Serialize;

use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::CandidateVersion;

/// The extent of one appended method invalidation.
///
/// **Why a closed sum and not a bare [`AttemptKey`].** The two things that are actually found unsound
/// after the fact have different extents. A single attempt can be invalidated — a run later shown to
/// have measured through a mistake specific to it. A *method* can be invalidated — the measured code
/// path itself was wrong, in which case every attempt of that candidate at that version is unsound,
/// including ones not yet written when the finding was made. Recording the second as N per-attempt
/// records would require enumerating the attempts, and would silently fail to cover any appended
/// later; recording it as one attempt key would understate it. So the extent is stated, and
/// [`Self::covers`] is a total match over it.
///
/// **Why the candidate version and not the harness commit.** It is the version bound into every
/// [`AttemptKey`], and the same reason applies: a formatting-only commit must not invalidate
/// evidence, and a semantic change made without a commit must not appear valid.
///
/// **Freely constructible, and that is correct.** This names *which* evidence a claim is about; it
/// asserts nothing about whether the claim is true. The record that carries it —
/// [`MethodSupersession`](super::method_supersession::MethodSupersession) — is likewise a recorded
/// human judgement rather than a derived finding, and neither pretends otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum SupersededScope {
    /// Exactly one attempt, retry ordinal included. A retry is a different attempt with its own
    /// evidence, so invalidating an original says nothing about its retry.
    Attempt(AttemptKey),
    /// Every attempt of one candidate at one implementation version — the extent of a finding about
    /// the measured code path rather than about a run.
    CandidateVersion {
        candidate: CandidateId,
        version: CandidateVersion,
    },
}

impl SupersededScope {
    /// Whether this scope reaches `attempt`.
    ///
    /// Implemented rather than stubbed because it is definitional, not a policy: an attempt scope
    /// covers exactly its own key, and a version scope covers exactly the attempts whose candidate
    /// and version it names. The *selection* rule that folds these over terminal outcomes is the
    /// deferred part, and it lives on
    /// `ReconciledCampaign`.
    pub(crate) fn covers(self, attempt: AttemptKey) -> bool {
        match self {
            Self::Attempt(superseded) => superseded == attempt,
            Self::CandidateVersion { candidate, version } => {
                attempt.candidate() == candidate && attempt.version() == version
            }
        }
    }
}
