//! Shared fixture: a real, fully accounted sixty-slot ledger that each test perturbs in one way.
//!
//! **The testability ceiling, stated once here because it shapes every file in this tree.**
//! [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance) has
//! exactly one constructor, and it reads a verified distribution, a running server, and a published
//! module artifact. So no ledger built in this process can contain a `Provisioned` line — and
//! because reconciliation requires one for every attempt that published, no ledger built here can
//! contain a `Complete` attempt either, nor a `Failed` one that got past publish. What these tests
//! therefore cannot reach is listed in [`super`], rather than papered over with a test-only
//! provenance seam: adding one would make the admission gate forgeable, which is the one thing the
//! sealed constructor exists to prevent.
//!
//! What remains is not a thin residue. Two outcome classes need no provisioned instance at all — a
//! preflight rejection, and a failure before publish — so [`full_campaign`] is a *genuinely
//! accepted* sixty-slot campaign rather than a stand-in.
//!
//! **What that baseline shows, and what it does not.** Accepted, it exercises the paths a healthy
//! ledger takes: contiguous sequences, an opening inventory, one terminal per predeclared original,
//! exactly one clearance before the one launched attempt's terminal line and none for the rest, and
//! no provisioned or post-attempt line anywhere — because no outcome in it published or measured.
//! It contains no retry and no supersession at all, and by being accepted it cannot show any
//! rejection: an orphan line or a dangling supersession is precisely what an accepted ledger does
//! not have. Retry, supersession, orphan and every refusal are covered by the tests that *augment
//! or perturb* this baseline in exactly one way each — the baseline supplies the healthy ledger,
//! and each test supplies the single difference whose consequence it names.
//!
//! Every value here comes from a real constructor. Nothing fabricates evidence: the failed outcome
//! carries genuinely empty [`PartialEvidence`], and both gates were recomputed by
//! [`PassedEnvironmentGate::cleared`] and [`FailedEnvironmentGate::refused`] from readings that
//! really do pass and really do fail.

use anyhow::Result;

use crate::observation::record_seq::RecordSeq;
use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::attempt_inventory::AttemptInventory;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
use crate::view_read_set_campaign::campaign_ledger_line::CampaignLedgerLine;
use crate::view_read_set_campaign::campaign_params::{
    ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
};
use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::candidate_id::CandidateId;
use crate::view_read_set_campaign::candidate_version::ENTITY_OWNER_SENDER_VIEW_VERSION;
use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::failed_environment_gate::FailedEnvironmentGate;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::partial_evidence::PartialEvidence;
use crate::view_read_set_campaign::passed_environment_gate::PassedEnvironmentGate;
use crate::view_read_set_campaign::pilot_block_index::PilotBlockIndex;
use crate::view_read_set_campaign::reconciled_campaign::ReconciledCampaign;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
use crate::view_read_set_campaign::scale_point::ScalePoint;
use crate::view_read_set_campaign::stage_repetition::StageRepetition;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

/// The frozen inventory, exactly as a running campaign freezes it.
///
/// Reachable in a test because freezing is pure: it derives a seeded permutation over frozen
/// constants and touches no server or file.
pub(super) fn inventory() -> AttemptInventory {
    AttemptInventory::frozen().expect("the frozen campaign inventory has sixty distinct slots")
}

/// The campaign's real pins, resolved exactly as a running campaign resolves them.
pub(super) fn provenance() -> CampaignProvenance {
    CampaignProvenance::resolved().expect("the frozen version and release-commit pins parse")
}

/// An attempt identity at the first block's first rung, in `role`, at `retry`.
///
/// Built from its coordinates rather than read back out of the inventory because a retry identity
/// has to be *minted*: it is not predeclared anywhere, since the inventory is frozen before any
/// outcome exists to earn one.
fn key(role: RunRole, retry: RetryOrdinal) -> AttemptKey {
    AttemptKey::new(
        CandidateId::EntityOwnerSenderView,
        ScalePoint::validated(ExperimentAxis::UnrelatedGlobalRows, 0)
            .expect("the frozen unrelated-global-rows ladder has a first rung"),
        role,
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

/// `key`, checked to be one the frozen inventory really predeclares.
///
/// Membership is asserted rather than assumed: if it did not hold, the ledgers below would quietly
/// stop covering the slot they name, and the tests would keep passing while testing less.
fn predeclared(key: AttemptKey) -> AttemptKey {
    inventory()
        .attempts()
        .iter()
        .copied()
        .find(|frozen| *frozen == key)
        .expect("the fixture's coordinates name a slot the frozen inventory predeclares")
}

/// The one slot of the fixture's campaign that *launched* and then failed.
///
/// Its presence is what makes the accepted ledger exercise the launched-attempt rules — a clearance
/// is required for this identity and for no other. Retry-ineligible: an application fault is a
/// measured answer about the candidate at every stage.
pub(super) fn failed_slot() -> AttemptKey {
    predeclared(key(RunRole::Arm, RetryOrdinal::ORIGINAL))
}

/// A slot whose original is refused by the preflight gate, and is therefore retry-eligible.
pub(super) fn eligible_slot() -> AttemptKey {
    predeclared(key(RunRole::Control, RetryOrdinal::ORIGINAL))
}

/// The retry identity of [`eligible_slot`] — the same logical slot at [`RetryOrdinal::RETRY`].
pub(super) fn eligible_retry() -> AttemptKey {
    key(RunRole::Control, RetryOrdinal::RETRY)
}

/// The retry identity of [`failed_slot`], whose original the retry rule refuses to reopen.
pub(super) fn ineligible_retry() -> AttemptKey {
    key(RunRole::Arm, RetryOrdinal::RETRY)
}

/// A ledger under construction, assigning each appended record the next campaign sequence.
///
/// Sequences are assigned rather than passed in, so these ledgers are contiguous by construction: a
/// sequence defect is something a test states outright rather than something it can introduce by
/// miscounting.
pub(super) struct Ledger {
    lines: Vec<CampaignLedgerLine>,
    next: RecordSeq,
}

impl Ledger {
    /// A ledger containing only its opening inventory line.
    pub(super) fn opened() -> Self {
        let mut ledger = Self {
            lines: Vec::new(),
            next: RecordSeq::zero(),
        };
        ledger.append(CampaignRecord::Inventory {
            inventory: inventory(),
            provenance: provenance(),
        });
        ledger
    }

    /// Append one record at the next sequence, and report the sequence it took.
    pub(super) fn append(&mut self, body: CampaignRecord) -> RecordSeq {
        let seq = self.next;
        self.lines.push(CampaignLedgerLine::at(seq, body));
        self.next = seq.next();
        seq
    }

    /// Append the terminal record for `attempt`, preceded by whatever auxiliary lines its outcome
    /// requires — a clearance for a launched attempt, and nothing else, since no outcome reachable
    /// in this process publishes or measures.
    pub(super) fn append_attempt(&mut self, attempt: AttemptKey, outcome: AttemptOutcome) {
        if launched(&outcome) {
            self.append(CampaignRecord::PreflightCleared {
                attempt,
                gate: passed_gate(),
            });
        }
        self.append(CampaignRecord::Terminal {
            record: terminal(attempt, outcome),
        });
    }

    /// The lines built so far.
    pub(super) fn lines(&self) -> Vec<CampaignLedgerLine> {
        self.lines.clone()
    }

    /// Reconcile the ledger as built.
    pub(super) fn reconciled(&self) -> Result<ReconciledCampaign> {
        ReconciledCampaign::reconciled(self.lines())
    }

    /// Reconcile the ledger as built, requiring acceptance.
    pub(super) fn accepted(&self) -> ReconciledCampaign {
        self.reconciled().expect(
            "the fixture's ledger accounts for every frozen slot exactly as the protocol requires",
        )
    }

    /// Reconcile the ledger as built, requiring refusal, and report the rendered refusal.
    pub(super) fn refused(&self, expectation: &str) -> String {
        let error = self.reconciled().expect_err(expectation);
        format!("{error:#}")
    }
}

/// The whole sixty-slot campaign, accounted exactly as the protocol requires.
///
/// One slot launches and fails before publish; every other is refused by the preflight gate. That
/// mix is deliberate rather than uniform: a ledger of nothing but rejections would never exercise
/// the clearance-required rule, leaving "exactly one clearance, before the terminal line" untested
/// in the accepted case.
pub(super) fn full_campaign() -> Ledger {
    campaign_omitting(None)
}

/// The same campaign with one predeclared slot's lines left out entirely, for the tests that need a
/// ledger which fails to account for a slot the inventory names.
pub(super) fn campaign_omitting(omitted: Option<AttemptKey>) -> Ledger {
    let mut ledger = Ledger::opened();
    let failed = failed_slot();
    for attempt in inventory().attempts() {
        if omitted == Some(*attempt) {
            continue;
        }
        let outcome = if *attempt == failed {
            failed_before_publish(*attempt)
        } else {
            preflight_rejected()
        };
        ledger.append_attempt(*attempt, outcome);
    }
    ledger
}

/// Rebuild `lines` with sequences reassigned contiguously from zero, preserving stream order.
///
/// For the tests that add or remove a line and want the *removal* to be what reconciliation
/// reports, rather than the hole in the sequence it would otherwise leave behind.
pub(super) fn resequenced(lines: &[CampaignLedgerLine]) -> Vec<CampaignLedgerLine> {
    let mut seq = RecordSeq::zero();
    let mut resequenced = Vec::with_capacity(lines.len());
    for line in lines {
        resequenced.push(CampaignLedgerLine::at(seq, line.body().clone()));
        seq = seq.next();
    }
    resequenced
}

/// Bind an identity to its terminal outcome through the sole constructor.
pub(super) fn terminal(attempt: AttemptKey, outcome: AttemptOutcome) -> TerminalAttemptRecord {
    TerminalAttemptRecord::sealed(attempt, outcome)
        .expect("the fixture's outcomes carry evidence measured at the identity's own scale point")
}

/// Whether an outcome's attempt launched, and therefore requires a preflight clearance.
fn launched(outcome: &AttemptOutcome) -> bool {
    match outcome {
        AttemptOutcome::Complete { .. } | AttemptOutcome::Failed { .. } => true,
        AttemptOutcome::PreflightRejected { .. } | AttemptOutcome::NotRun { .. } => false,
    }
}

/// A retained diagnostic. Opaque to every accounting rule, which reads the outcome's variant and
/// its typed lifecycle facts rather than this text.
fn diagnostic() -> DiagnosticArtifact {
    DiagnosticArtifact::of_error(&anyhow::anyhow!("the fixture's terminating condition"))
}

/// Two readings the frozen separation apart on a four-CPU host, differing only in the second's
/// load.
///
/// The load is the fixture's single lever: available RAM, swap-out delta and memory pressure are
/// inside the gate in both readings, so whether the pair passes is decided by whether the load
/// falls and by nothing incidental.
fn gate_evidence(second_load_centi: u64) -> EnvironmentGateEvidence {
    let first = EnvironmentSample::observed(0, 100, ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, 0, 0);
    let second = EnvironmentSample::observed(
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
        second_load_centi,
        ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        0,
        0,
    );
    EnvironmentGateEvidence::paired(first, second, 4)
        .expect("the two readings are the frozen separation apart on a host reporting four CPUs")
}

/// A clearance derived from readings whose load genuinely falls.
pub(super) fn passed_gate() -> PassedEnvironmentGate {
    PassedEnvironmentGate::cleared(gate_evidence(50))
        .expect("a load that falls well under four CPUs passes every clause of the gate")
}

/// One host reading, of the kind taken immediately after a measured attempt.
pub(super) fn post_attempt_sample() -> EnvironmentSample {
    EnvironmentSample::observed(
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
        50,
        ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        0,
        0,
    )
}

/// An attempt the prospective gate refused to launch, because the load did not fall.
///
/// Retry-*eligible*: nothing was measured, so retrying references no measured outcome.
pub(super) fn preflight_rejected() -> AttemptOutcome {
    AttemptOutcome::PreflightRejected {
        gate: FailedEnvironmentGate::refused(gate_evidence(100))
            .expect("a load that does not fall between the two samples fails the gate"),
        diagnostic: diagnostic(),
    }
}

/// A launched attempt that failed before publishing, terminated by an application fault.
///
/// Requires exactly one preflight clearance, no provisioned line — it never published — and no
/// post-attempt reading, since it measured nothing.
pub(super) fn failed_before_publish(attempt: AttemptKey) -> AttemptOutcome {
    AttemptOutcome::Failed {
        kind: FailureKind::Application,
        stage: FailureStage::BeforePublish,
        partial: PartialEvidence::sealed(attempt.scale(), Vec::new(), None)
            .expect("an empty channel prefix with no composition finding is partial evidence"),
        diagnostic: diagnostic(),
    }
}
