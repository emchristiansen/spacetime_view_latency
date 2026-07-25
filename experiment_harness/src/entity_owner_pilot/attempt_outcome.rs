//! The one terminal disposition of a predeclared attempt.

use serde::Serialize;

use crate::entity_owner_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact;
use crate::entity_owner_pilot::failure_kind::FailureKind;
use crate::entity_owner_pilot::method_validity::MethodValidity;
use crate::entity_owner_pilot::not_run_reason::NotRunReason;
use crate::entity_owner_pilot::partial_evidence::PartialEvidence;

/// The single terminal disposition of one predeclared attempt, transcribed from the spec's "Minimal
/// type design" `AttemptOutcome`.
///
/// The Parked Frontier requires "exactly one terminal Complete/Failed/NotRun record for each of the
/// ten predeclared attempts". Modelling that as one closed enum — rather than as separate optional
/// fields on a record — is what makes a half-recorded attempt unrepresentable: an attempt cannot be
/// both complete and failed, cannot be complete without an artifact, and cannot be failed without a
/// diagnostic, because each variant carries exactly the payload its disposition requires.
///
/// The payload types are what carry the guarantees: only a full ladder walk can seal an
/// [`EvidenceArtifact`], and only a strict prefix can seal a [`PartialEvidence`], so the two
/// evidence-bearing variants cannot be assembled from each other's data.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum AttemptOutcome {
    /// The attempt walked the entire frozen ladder. Carries the complete artifact and whether its
    /// method is still admissible.
    Complete {
        artifact: EvidenceArtifact,
        validity: MethodValidity,
    },
    /// The attempt terminated partway. Carries the classified cause, whatever rungs had already
    /// completed, and the retained diagnostic chain — the spec's "failed with partial evidence and
    /// diagnostics".
    Failed {
        kind: FailureKind,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    },
    /// The attempt was predeclared but never executed, with the reason it was skipped.
    NotRun { reason: NotRunReason },
}
