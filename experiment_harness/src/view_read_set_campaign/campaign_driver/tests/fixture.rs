//! Shared fixture: one attempt identity at either ordinal, the terminal outcomes retry scheduling
//! reads, and one payload of each kind the recording adapters append.
//!
//! Every value comes from a real constructor. Nothing fabricates evidence: the failed outcome
//! carries genuinely empty [`PartialEvidence`], and the preflight rejection carries a gate that
//! [`FailedEnvironmentGate::refused`] recomputed and found failing. Retry scheduling reads only the
//! outcome's variant, cause and stage — never a payload's contents — so these are the complete
//! inputs rather than a reduced stand-in.
//!
//! This tree keeps its own fixture rather than sharing
//! [`TerminalAttemptRecord`](crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord)'s,
//! whose helpers are `pub(super)` and so reach only that module's own tests. The overlap is real and
//! deliberate: a shared fixture would couple the eligibility rule's tests to the scheduler's, and
//! one tree's convenience change would silently move the other's inputs.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_inventory::AttemptInventory;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
use crate::view_read_set_campaign::campaign_params::{
    ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
};
use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
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
use crate::view_read_set_campaign::passed_environment_gate::PassedEnvironmentGate;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

/// The one scale point every record here is filed under. Which rung it is does not enter retry
/// scheduling; it only has to be the *same* one any retained evidence was measured at, since
/// [`TerminalAttemptRecord::sealed`] refuses a mismatch.
fn scale() -> ScalePoint {
    ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 0)
        .expect("the frozen unrelated-global-rows ladder has a first rung")
}

/// An attempt identity at `retry`, identical in every other coordinate.
pub(super) fn key(retry: RetryOrdinal) -> AttemptKey {
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

/// A retained diagnostic. Opaque to retry scheduling, which reads the classified outcome rather than
/// this text.
fn diagnostic() -> DiagnosticArtifact {
    DiagnosticArtifact::of_error(&anyhow::anyhow!("the fixture's terminating condition"))
}

/// An attempt the prospective gate refused to launch — the categorically retry-eligible outcome.
///
/// The readings fail exactly one clause, the second sample's load not falling below the first's,
/// with available RAM, swap flow and memory pressure all inside the gate. The refusal is therefore a
/// real derived one, produced by the load clause.
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

/// An attempt terminated by an application fault, retaining nothing measured.
///
/// Ineligible at every stage: an application fault is a measured answer about the candidate, so
/// rerunning it would be rerunning the experiment rather than recovering from an accident.
pub(super) fn application_failure() -> AttemptOutcome {
    AttemptOutcome::Failed {
        kind: FailureKind::Application,
        stage: FailureStage::BeforePublish,
        partial: PartialEvidence::sealed(scale(), Vec::new(), None)
            .expect("an empty channel prefix with no composition finding is partial evidence"),
        diagnostic: diagnostic(),
    }
}

/// The frozen sixty-slot order the campaign's opening line records.
pub(super) fn inventory() -> AttemptInventory {
    AttemptInventory::frozen().expect("the frozen campaign inventory has sixty distinct slots")
}

/// The campaign-constant pins the opening line records alongside the inventory.
pub(super) fn campaign_provenance() -> CampaignProvenance {
    CampaignProvenance::resolved().expect("the frozen version and release-commit pins parse")
}

/// A clearance that really passes: the same readings as [`preflight_rejected`]'s except that the
/// second sample's load falls, which is the one clause that refusal turns on.
pub(super) fn passed_gate() -> PassedEnvironmentGate {
    let first = EnvironmentSample::observed(0, 100, ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, 0, 0);
    let second = EnvironmentSample::observed(
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
        50,
        ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        0,
        0,
    );
    let evidence = EnvironmentGateEvidence::paired(first, second, 4)
        .expect("the two readings are the frozen separation apart on a host reporting four CPUs");

    PassedEnvironmentGate::cleared(evidence)
        .expect("a load that falls well under four CPUs passes every clause of the gate")
}

/// The host reading taken immediately after a measured attempt.
pub(super) fn post_attempt_sample() -> EnvironmentSample {
    EnvironmentSample::observed(
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
        50,
        ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        0,
        0,
    )
}

/// A predeclared attempt that never executed, because a preceding attempt's release failed.
///
/// Raises no retry question at all: there is no measured slot to reopen.
pub(super) fn not_run() -> AttemptOutcome {
    AttemptOutcome::NotRun {
        reason: NotRunReason::PriorAttemptReleaseFailed {
            diagnostic: diagnostic(),
        },
    }
}
