//! A failed attempt, with everything needed to judge and diagnose it.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::failure_phase::FailurePhase;
use crate::control_registry_discovery_screen::partial_evidence::PartialEvidence;
use crate::control_registry_discovery_screen::retry_eligibility::RetryEligibility;
use crate::control_registry_discovery_screen::sampling_progress::SamplingProgress;

/// One attempt's failure: its typed kind, how far it had got, what it had observed, the retained
/// diagnostic, and the retry eligibility those imply.
///
/// The spec's retry rule is two-dimensional, and both dimensions are stored because neither implies
/// the other. `Reducer` is pre-sample yet never retryable, because it is an application error;
/// `Subscription` is infrastructure-adjacent yet never retryable, because sampling has begun. A
/// single field could not express both cases without one of them being wrong.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AttemptFailure {
    kind: FailureKind,
    progress: SamplingProgress,
    retry_eligibility: RetryEligibility,
    partial: PartialEvidence,
    diagnostic: DiagnosticArtifact,
}

impl AttemptFailure {
    /// Record a failure, rejecting an impossible ordering and deriving retry eligibility.
    ///
    /// Eligibility is computed here rather than accepted from the caller, so no attempt can declare
    /// itself retryable. It is `Retryable` exactly when the failure was infrastructural *and* struck
    /// before the first measured sample — the spec's rule verbatim.
    pub(crate) fn observed(
        kind: FailureKind,
        progress: SamplingProgress,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    ) -> Result<Self> {
        ensure!(
            kind.can_follow_first_sample() || progress == SamplingProgress::BeforeFirstSample,
            "{kind:?} strictly precedes the timed subscription, so it cannot be recorded as \
             occurring at or after the first measured sample",
        );
        let observed = match partial {
            PartialEvidence::NothingObserved => None,
            PartialEvidence::ObservedComposition { composition } => Some(composition),
        };
        ensure!(
            observed.is_none() || kind.can_observe_composition(),
            "{kind:?} strikes before the subscription applies, so it cannot carry an observed \
             composition",
        );
        ensure!(
            observed.is_none() || progress == SamplingProgress::AtOrAfterFirstSample,
            "an observed composition means the subscription applied, so the attempt cannot also \
             report that it failed before its first measured sample",
        );
        ensure!(
            !kind.requires_observed_composition() || observed.is_some(),
            "{kind:?} is an observed-versus-expected composition mismatch, so it cannot report that \
             nothing was observed",
        );
        // A semantic failure *is* the mismatch. If every cache matched, whatever went wrong was not
        // a composition failure, and recording it as one would put a passing composition check into
        // the ledger under a semantic verdict.
        ensure!(
            kind != FailureKind::Semantics
                || observed.is_some_and(|composition| !composition.matches()),
            "a Semantics failure must carry the composition mismatch it names, but every cache \
             matched its frozen expectation",
        );
        let retry_eligibility = match (kind.phase(), progress) {
            (FailurePhase::Infrastructure, SamplingProgress::BeforeFirstSample) => {
                RetryEligibility::Retryable
            }
            (FailurePhase::Infrastructure, SamplingProgress::AtOrAfterFirstSample)
            | (FailurePhase::ApplicationOrSemantic, _) => RetryEligibility::NotRetryable,
        };
        Ok(Self {
            kind,
            progress,
            retry_eligibility,
            partial,
            diagnostic,
        })
    }
}
