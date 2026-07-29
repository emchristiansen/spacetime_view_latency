//! A failed attempt, with everything needed to judge and diagnose it.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::control_registry_discovery_screen::attempt_stage::AttemptStage;
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::failure_phase::FailurePhase;
use crate::control_registry_discovery_screen::partial_evidence::PartialEvidence;
use crate::control_registry_discovery_screen::retry_eligibility::RetryEligibility;
use crate::control_registry_discovery_screen::sampling_progress::SamplingProgress;

/// One attempt's failure: its typed kind, the record shape that kind's evidence supports, how far it
/// had got, what it retained, the diagnostic, and the retry eligibility those imply.
///
/// Stage, phase, and sampling progress are all **derived** from the kind and then stored. Derived,
/// so no caller can describe a failure as having reached further than its kind allows; stored, so a
/// reader holding only the ledger sees the classification that was actually in force rather than
/// having to recompute it against whatever the code says today.
///
/// **No `Debug`**, inherited from the retained [`PartialEvidence`]: see
/// [`RejectedApplyNanos`](super::rejected_apply_nanos::RejectedApplyNanos) for why a textual
/// rendering of a rejected duration is withheld. Diagnostics use the typed accessors and the
/// [`DiagnosticArtifact`] instead, which lose nothing.
#[derive(Clone, Serialize)]
pub(crate) struct AttemptFailure {
    kind: FailureKind,
    stage: AttemptStage,
    phase: FailurePhase,
    progress: SamplingProgress,
    retry_eligibility: RetryEligibility,
    partial: PartialEvidence,
    diagnostic: DiagnosticArtifact,
}

impl AttemptFailure {
    /// Record a failure, rejecting every shape its kind cannot have and deriving the rest.
    ///
    /// **Sampling progress is not a parameter.** It is exactly whether the kind's stage is inside
    /// the measurement window, so accepting it from a caller could only introduce a disagreement
    /// with the kind — the two can no longer contradict each other because there is only one of
    /// them.
    ///
    /// Retry eligibility is likewise computed here, so no attempt can declare itself retryable. It
    /// is `Retryable` exactly when the failure was infrastructural *and* struck before the first
    /// measured sample — the spec's rule verbatim, which leaves `Provision` and `Connect` as the
    /// only retryable kinds.
    pub(crate) fn observed(
        kind: FailureKind,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    ) -> Result<Self> {
        // The biconditional that makes raw-sample retention structural: a kind reachable only past
        // a completed timed apply must carry its nanoseconds, and a kind that strictly precedes one
        // must not invent them.
        ensure!(
            kind.requires_sample() == partial.has_sample(),
            "{kind:?} {} a completed timed apply behind it, but the retained partial evidence {} a \
             raw sample",
            if kind.requires_sample() {
                "always has"
            } else {
                "never has"
            },
            if partial.has_sample() {
                "carries"
            } else {
                "carries none"
            },
        );

        let composition = partial.composition();
        ensure!(
            composition.is_none() || kind.can_observe_composition(),
            "{kind:?} strikes before the four caches are read, so it cannot carry an observed \
             composition",
        );
        ensure!(
            !kind.requires_observed_composition() || composition.is_some(),
            "{kind:?} is an observed-versus-expected composition mismatch, so it cannot report that \
             nothing was observed",
        );
        // A semantic failure *is* the mismatch. If every cache matched, whatever went wrong was not
        // a composition failure, and recording it as one would put a passing composition check into
        // the ledger under a semantic verdict.
        ensure!(
            kind != FailureKind::Semantics
                || composition.is_some_and(|composition| !composition.matches()),
            "a Semantics failure must carry the composition mismatch it names, but every cache \
             matched its frozen expectation",
        );

        let progress = match kind.can_follow_first_sample() {
            true => SamplingProgress::AtOrAfterFirstSample,
            false => SamplingProgress::BeforeFirstSample,
        };
        let phase = kind.phase();
        let retry_eligibility = match (phase, progress) {
            (FailurePhase::Infrastructure, SamplingProgress::BeforeFirstSample) => {
                RetryEligibility::Retryable
            }
            (FailurePhase::Infrastructure, SamplingProgress::AtOrAfterFirstSample)
            | (FailurePhase::ApplicationOrSemantic, _)
            | (FailurePhase::HarnessObservation, _) => RetryEligibility::NotRetryable,
        };

        Ok(Self {
            kind,
            stage: kind.stage(),
            phase,
            progress,
            retry_eligibility,
            partial,
            diagnostic,
        })
    }

    /// What kind of failure this was.
    pub(crate) fn kind(&self) -> FailureKind {
        self.kind
    }

    /// The record shape this failure's evidence supports.
    pub(crate) fn stage(&self) -> AttemptStage {
        self.stage
    }

    /// Whether the first measured sample had begun.
    pub(crate) fn progress(&self) -> SamplingProgress {
        self.progress
    }

    /// Whether the spec's retry rule admits another attempt at this slot.
    pub(crate) fn retry_eligibility(&self) -> RetryEligibility {
        self.retry_eligibility
    }
}
