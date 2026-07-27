//! Shared fixture: one attempt identity, and the light terminal outcomes the retry rule reads.
//!
//! Every outcome here is built through its real constructor. Nothing fabricates evidence: the failed
//! outcomes carry genuinely empty [`PartialEvidence`], and the preflight rejection carries a gate
//! that [`FailedEnvironmentGate::refused`] recomputed and found failing. The retry rule reads the
//! outcome's *variant, cause and stage* and never its payload's contents, so these are the complete
//! inputs rather than a reduced stand-in.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
use crate::view_read_set_campaign::campaign_params::{
    ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
};
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::failed_environment_gate::FailedEnvironmentGate;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::not_run_reason::NotRunReason;
use crate::view_read_set_campaign::partial_evidence::PartialEvidence;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

/// The one scale point every record in this tree is filed under. Which rung it is does not enter the
/// retry rule; it only has to be the *same* one the retained evidence was measured at, since
/// [`TerminalAttemptRecord::sealed`] refuses a mismatch.
fn scale() -> ScalePoint {
    ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 0)
        .expect("the frozen unrelated-global-rows ladder has a first rung")
}

/// An attempt identity at `retry`, identical in every other coordinate.
fn key(retry: RetryOrdinal) -> AttemptKey {
    AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        scale(),
        RunRole::Arm,
        StageRepetition::Pilot(
            PilotBlockIndex::ALL
                .first()
                .copied()
                .expect("the Pilot stage has five frozen blocks"),
        ),
        retry,
        ENTITY_OWNER_SENDER_VIEW_VERSION,
    )
}

/// Bind `outcome` to an identity at `retry`, through the sole constructor.
pub(super) fn record(retry: RetryOrdinal, outcome: AttemptOutcome) -> TerminalAttemptRecord {
    TerminalAttemptRecord::sealed(key(retry), outcome)
        .expect("the fixture's outcomes carry evidence measured at the identity's own scale point")
}

/// A retained diagnostic. Opaque to the retry rule, which classifies on
/// [`FailureKind`] rather than on this text.
fn diagnostic() -> DiagnosticArtifact {
    DiagnosticArtifact::of_error(&anyhow::anyhow!("the fixture's terminating condition"))
}

/// A failed attempt terminated by `kind` at `stage`, retaining nothing measured.
///
/// The empty prefix is the honest shape for these cases: an attempt that failed before its first
/// sample has no channel to retain, and the rule's own timing term comes from `stage` rather than
/// from the prefix length, which cannot distinguish the two sides of it.
pub(super) fn failed(kind: FailureKind, stage: FailureStage) -> AttemptOutcome {
    AttemptOutcome::Failed {
        kind,
        stage,
        partial: PartialEvidence::sealed(scale(), Vec::new(), None)
            .expect("an empty channel prefix with no composition finding is partial evidence"),
        diagnostic: diagnostic(),
    }
}

/// An attempt the prospective gate refused to launch.
///
/// The readings fail exactly one clause — the second sample's load does not fall below the first's —
/// with available RAM, swap flow and memory pressure all inside the gate. So the refusal is a real
/// derived one, and it is the load clause that produced it.
pub(super) fn preflight_rejected() -> AttemptOutcome {
    let first = EnvironmentSample::observed(0, 100, ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, 0, 0);
    let second = EnvironmentSample::observed(
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
        100,
        ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        0,
        0,
    );
    let evidence = EnvironmentGateEvidence::paired(first, second, 4)
        .expect("the two readings are the frozen separation apart on a host reporting four CPUs");

    AttemptOutcome::PreflightRejected {
        gate: FailedEnvironmentGate::refused(evidence)
            .expect("a load that does not fall between the two samples fails the gate"),
        diagnostic: diagnostic(),
    }
}

/// An attempt whose prospective preflight could not be read, so no gate verdict exists.
///
/// Carries a diagnostic and nothing else, which is the variant's whole shape: there are no readings
/// to pair, so nothing here stands in for a gate.
pub(super) fn preflight_unreadable() -> AttemptOutcome {
    AttemptOutcome::PreflightUnreadable {
        diagnostic: diagnostic(),
    }
}

/// A predeclared attempt that never executed, because a preceding attempt's release failed.
pub(super) fn not_run() -> AttemptOutcome {
    AttemptOutcome::NotRun {
        reason: NotRunReason::PriorAttemptReleaseFailed {
            diagnostic: diagnostic(),
        },
    }
}
