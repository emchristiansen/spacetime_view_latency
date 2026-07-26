//! The one terminal disposition of a predeclared attempt.

use serde::Serialize;

use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::evidence_artifact::EvidenceArtifact;
use crate::view_read_set_campaign::failed_environment_gate::FailedEnvironmentGate;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::measured_sample_boundary::MeasuredSampleBoundary;
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
/// [`FailureKind::Infrastructure`] together with the first-measured-sample boundary; everything else
/// is ineligible. A variant added here without a retry disposition fails to compile in that
/// function rather than silently defaulting.
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
    /// `boundary` is stated by the driver rather than inferred from `partial`, because an empty
    /// channel prefix cannot distinguish a failure before any measurement from one during the very
    /// first sample. It is serialized with the rest so the retry decision is auditable from the
    /// ledger alone.
    Failed {
        kind: FailureKind,
        boundary: MeasuredSampleBoundary,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    },
    /// Predeclared but never executed.
    NotRun { reason: NotRunReason },
}
