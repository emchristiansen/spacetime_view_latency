//! A failed attempt, with everything needed to judge and diagnose it.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::attempt_stage::AttemptStage;
use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::indexed_sender_view_calibration_pilot::failure_kind::FailureKind;
use crate::indexed_sender_view_calibration_pilot::failure_phase::FailurePhase;
use crate::indexed_sender_view_calibration_pilot::partial_evidence::PartialEvidence;
use crate::indexed_sender_view_calibration_pilot::retry_eligibility::RetryEligibility;
use crate::indexed_sender_view_calibration_pilot::sampling_progress::SamplingProgress;

/// One attempt's failure: its typed kind, the record shape that kind's evidence supports, how far it
/// had got, what it retained, the diagnostic, and the retry eligibility those imply.
///
/// Stage, phase, and sampling progress are all **derived** from the kind and then stored. Derived,
/// so no caller can describe a failure as having reached further than its kind allows; stored, so a
/// reader holding only the ledger sees the classification that was actually in force rather than
/// having to recompute it against whatever the code says today.
///
/// **No `Debug`**, inherited from the retained [`PartialEvidence`].
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
    /// with the kind.
    ///
    /// Retry eligibility is likewise computed here, so no attempt can declare itself retryable. It
    /// is `Retryable` exactly when the failure was infrastructural *and* struck before the first
    /// measured sample — the spec's rule verbatim, which leaves `Provision` and `Connect` as the
    /// only retryable kinds.
    ///
    /// **The series invariant is a pair of implications, not a biconditional**, and that is the one
    /// place this model departs from the discovery screen's. A cold apply either happened or did
    /// not, so E3 can insist a kind carries a sample exactly when it requires one. A paced batch is
    /// up to a thousand appends, so a batch that stops partway holds a genuine partial series — see
    /// [`FailureKind::permits_series`]. What is still forbidden is a kind that strictly precedes the
    /// first append reporting samples it could not have taken, and a post-batch kind reporting none.
    pub(crate) fn observed(
        kind: FailureKind,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    ) -> Result<Self> {
        ensure!(
            !partial.has_series() || kind.permits_series(),
            "{kind:?} strikes before the first append, so it cannot carry a retained series",
        );
        ensure!(
            !kind.requires_series() || partial.has_series(),
            "{kind:?} always has a paced batch that returned behind it, so it must retain that \
             batch's samples — which is not the same as reaching the frozen count, since a batch \
             that returned short is exactly what a Sample failure reports",
        );

        let mismatch = partial.mismatch();
        ensure!(
            mismatch.is_none() || kind.can_observe_composition(),
            "{kind:?} strikes before the caches are read, so it cannot carry a composition mismatch",
        );
        ensure!(
            !kind.requires_observed_composition() || mismatch.is_some(),
            "{kind:?} is an observed-versus-expected composition mismatch, so it cannot report that \
             nothing was observed",
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

    /// How many samples this failure retained, when its batch ran at all.
    pub(crate) fn retained_samples(&self) -> Option<usize> {
        self.partial.retained_samples()
    }
}
