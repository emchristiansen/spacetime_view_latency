//! The implementation version of the candidate an attempt exercised.

use serde::Serialize;

/// The monotonically increasing implementation version of a candidate's measured code path.
///
/// Binding it into every attempt identity is what stops evidence produced by materially different
/// code from being pooled as one sample. It is deliberately not derived from the git commit: a
/// formatting-only commit must not invalidate evidence, and a semantic change made without a commit
/// must not appear valid. Commit provenance is recorded separately by the run's build provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct CandidateVersion(u32);

/// The current implementation version of the `EntityOwnerSenderView` candidate.
///
/// Bump when the measured path's semantics change — the view body, the seeded composition, the
/// subscription shape, or what the measured interval covers — and never for a formatting or
/// comment-only change.
pub(crate) const ENTITY_OWNER_SENDER_VIEW_VERSION: CandidateVersion = CandidateVersion(1);
