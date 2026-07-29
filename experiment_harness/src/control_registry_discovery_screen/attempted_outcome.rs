//! How an attempt that actually ran ended.

use serde::Serialize;

use crate::control_registry_discovery_screen::attempt_failure::AttemptFailure;
use crate::control_registry_discovery_screen::cold_apply_evidence::ColdApplyEvidence;

/// The terminal outcome of an attempt that provisioned and ran: it either sealed evidence or failed.
///
/// Deliberately excludes "not run". A slot that never ran has no provision provenance and no host
/// observations, so it cannot inhabit the same shape as one that did — see
/// [`ScreenRecord`](super::screen_record::ScreenRecord), where the two are separate variants rather
/// than one variant with optional fields. That split is what makes "complete or failed implies fully
/// observed" a structural property instead of a convention.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum AttemptedOutcome {
    /// The attempt measured its target and its composition validated.
    Complete { evidence: ColdApplyEvidence },
    /// The attempt ran and did not produce valid evidence.
    Failed { failure: AttemptFailure },
}
