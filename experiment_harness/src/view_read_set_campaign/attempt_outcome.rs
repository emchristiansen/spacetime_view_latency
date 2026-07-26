//! The one terminal disposition of a predeclared attempt.

use serde::Serialize;

use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::evidence_artifact::EvidenceArtifact;
use crate::view_read_set_campaign::failed_environment_gate::FailedEnvironmentGate;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::method_validity::MethodValidity;
use crate::view_read_set_campaign::not_run_reason::NotRunReason;
use crate::view_read_set_campaign::partial_evidence::PartialEvidence;

/// The single terminal disposition of one predeclared attempt, per the spec's `AttemptOutcome`.
///
/// One closed enum rather than optional fields on a record: an attempt cannot be both complete and
/// failed, complete without an artifact, or failed without a diagnostic, because each variant
/// carries exactly the payload its disposition requires. The payload types carry the rest of the
/// guarantee — see [`EvidenceArtifact`] and [`PartialEvidence`].
///
/// **Why the preflight rejection is its own variant.** It is not a kind of failure of a run, because
/// no run happened: the gate is prospective and ends immediately before launch. Folding it into
/// [`Self::Failed`] would give it a [`PartialEvidence`] field, and "the preflight refused, and here
/// is the evidence it measured" is a state with no meaning. Having its own variant with no evidence
/// field at all makes that unrepresentable rather than merely unused.
///
/// **This is what the retry rule total-matches over.** [`Self::PreflightRejected`] is categorically
/// eligible; [`Self::Failed`] is eligible only for
/// [`FailureKind::Infrastructure`] together with a stage whose
/// [`MeasuredSampleBoundary`](super::measured_sample_boundary::MeasuredSampleBoundary) is
/// `BeforeFirst`; everything else is ineligible. A variant added here without a retry disposition
/// fails to compile in that function rather than silently defaulting.
///
/// **It is also what the ledger's per-attempt line rules total-match over**, since which auxiliary
/// lines an attempt must have is fixed by its disposition: a launched attempt has exactly one
/// preflight clearance, a published one has a provisioned provenance, and a measured one has exactly
/// one post-attempt reading. Those rules are stated where they are checked, in
/// `ReconciledCampaign::reconciled`.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum AttemptOutcome {
    /// Measured all four channels at its scale point and passed composition validation.
    Complete {
        artifact: EvidenceArtifact,
        validity: MethodValidity,
    },
    /// The prospective preflight gate refused to launch this attempt. No measurement began, so there
    /// is nothing measured to retain — only the readings that refused it.
    PreflightRejected {
        gate: FailedEnvironmentGate,
        diagnostic: DiagnosticArtifact,
    },
    /// Launched, then terminated before completing, retaining whatever channels and composition
    /// finding it had produced.
    ///
    /// `stage` is stated by the driver rather than inferred from `partial`, because an empty channel
    /// prefix cannot distinguish a failure before any measurement from one during the very first
    /// sample, and no cause can say whether the module had been published. It is serialized with the
    /// rest, and both the retry timing term
    /// ([`MeasuredSampleBoundary`]) and the "must a `Provisioned` line exist for this attempt?" rule
    /// are total matches over it — so neither decision is auditable only by trusting the driver's
    /// classification of the cause.
    Failed {
        kind: FailureKind,
        stage: FailureStage,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    },
    /// Predeclared but never executed.
    NotRun { reason: NotRunReason },
}
