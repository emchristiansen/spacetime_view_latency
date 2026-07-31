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
/// **Known gap, deliberately not fixed here: the typed physical cause is not recoverable.**
/// [`FailurePhase`] is a settlement-policy bucket, and [`FailureKind`] narrows a failure only to a
/// lifecycle location and category — `FailureKind::PacedBatch` says the attempt stopped in the paced
/// batch, not whether a reducer refused, a barrier exceeded its deadline, or a channel dropped.
/// Below that granularity the retained diagnostic *text* is the only causal detail kept, and nothing
/// typed distinguishes those cases. That is a real loss of structure, shared verbatim with the
/// accepted E3 screen rather than introduced by this pilot — remedying it means a coherent redesign
/// across both modules, which is a follow-up, not this module's work. It cannot affect this pilot's
/// only admissible output: every failure blocks the attempt and therefore blocks `W` selection
/// identically, whatever caused it.
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
    /// **Both evidence invariants are biconditionals.** A kind carries a retained series exactly when
    /// it can follow the first append, and carries a composition mismatch exactly when it *is* a
    /// composition mismatch. Neither direction is slack:
    ///
    /// - a kind that strictly precedes the first append cannot report samples it could not have
    ///   taken, and a kind that follows it cannot report none — partiality is an empty or short
    ///   [`RejectedSeries`](super::rejected_series::RejectedSeries), never an absent one;
    /// - a `Semantics` failure cannot claim nothing was observed, and no *other* kind may carry a
    ///   mismatch — which would file a semantic divergence under a label that names a different
    ///   fault.
    pub(crate) fn observed(
        kind: FailureKind,
        partial: PartialEvidence,
        diagnostic: DiagnosticArtifact,
    ) -> Result<Self> {
        ensure!(
            !partial.has_series() || kind.requires_series(),
            "{kind:?} strikes before the first append, so it cannot carry a retained series",
        );
        ensure!(
            !kind.requires_series() || partial.has_series(),
            "{kind:?} can follow the first append, so it must retain that batch's ordered samples — \
             an empty RejectedSeries when the batch failed on its very first append, never \
             NothingObserved, which claims the batch was never reached",
        );

        let mismatch = partial.mismatch();
        ensure!(
            mismatch.is_none() || kind.requires_observed_composition(),
            "{kind:?} is not an observed-versus-expected composition mismatch, so it cannot carry \
             one; filing a semantic failure under another kind hides which rows diverged behind a \
             label that says something else went wrong",
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

#[cfg(test)]
mod tests;
