//! The one terminal disposition of a predeclared attempt.

use serde::Serialize;

use crate::entity_owner_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact;
use crate::entity_owner_pilot::failure_kind::FailureKind;
use crate::entity_owner_pilot::method_validity::MethodValidity;
use crate::entity_owner_pilot::not_run_reason::NotRunReason;
use crate::entity_owner_pilot::partial_evidence::PartialEvidence;

/// The single terminal disposition of one predeclared attempt, per the spec's `AttemptOutcome`.
///
/// One closed enum rather than optional fields on a record: an attempt cannot be both complete and
/// failed, complete without an artifact, or failed without a diagnostic, because each variant
/// carries exactly the payload its disposition requires. The payload types carry the rest of the
/// guarantee — see [`EvidenceArtifact`] and [`PartialEvidence`].
#[derive(Debug, Clone, Serialize)]
pub(crate) enum AttemptOutcome {
    /// Walked the entire frozen ladder.
    Complete {
        artifact: EvidenceArtifact,
        validity: MethodValidity,
    },
    /// Terminated partway, retaining whatever rungs had completed.
    Failed {
        kind: FailureKind,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    },
    /// Predeclared but never executed.
    NotRun { reason: NotRunReason },
}
