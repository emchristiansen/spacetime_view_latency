//! The implementation version of the candidate an attempt exercised.

use serde::Serialize;

/// The monotonically increasing implementation version of a candidate's measured code path.
///
/// The spec requires that "analysis records must bind candidate version, runtime, ladder version,
/// stage, and result shape so stale or mismatched results cannot join". This is that binding: two
/// attempts whose candidate implementation differs carry different versions, so a later analysis
/// cannot silently pool evidence produced by materially different code as though it were one sample.
///
/// Bumping it is a deliberate act performed when a candidate's measured semantics change — see
/// [`ENTITY_OWNER_SENDER_VIEW_VERSION`]. It is *not* derived from the git commit: a formatting-only
/// commit must not invalidate evidence, and a semantic change made without a commit must not appear
/// valid. Commit provenance is recorded separately by the run's build provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct CandidateVersion(u32);

impl CandidateVersion {
    /// The version this attempt's candidate implementation is at.
    pub(crate) fn get(self) -> u32 {
        self.0
    }
}

/// The current implementation version of the `EntityOwnerSenderView` candidate.
///
/// Version 1 is the first measured implementation: the module's `entity_owner_sender_view` bare
/// sender-equality query-builder filter over the `(entity_uuid primary key, owner indexed)`
/// composition, as landed for the Smoke milestone. Bump this whenever the measured path's semantics
/// change — the view body, the seeded composition, the subscription shape, or what the measured
/// interval covers — and never for a formatting or comment-only change.
pub(crate) const ENTITY_OWNER_SENDER_VIEW_VERSION: CandidateVersion = CandidateVersion(1);
