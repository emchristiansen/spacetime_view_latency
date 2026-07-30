//! One attempt's durable ledger line.

use anyhow::Result;
use serde::Serialize;

use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;
use crate::indexed_sender_view_calibration_pilot::attempt_failure::AttemptFailure;
use crate::indexed_sender_view_calibration_pilot::attempt_key::AttemptKey;
use crate::indexed_sender_view_calibration_pilot::attempt_stage::AttemptStage;
use crate::indexed_sender_view_calibration_pilot::attempted_outcome::AttemptedOutcome;
use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::indexed_sender_view_calibration_pilot::host_observations::HostObservations;
use crate::indexed_sender_view_calibration_pilot::method_facts::MethodFacts;
use crate::indexed_sender_view_calibration_pilot::not_run_reason::NotRunReason;
use crate::indexed_sender_view_calibration_pilot::partial_provision::PartialProvision;
use crate::indexed_sender_view_calibration_pilot::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::indexed_sender_view_calibration_pilot::resource_disposition::ResourceDisposition;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// Whether a record shape past publication owned a releasable resource. Always, and unconditionally:
/// reaching publication means the staged module tempfile and the started server with its fresh data
/// directory were both the driver's to release, so every post-publication shape rejects
/// [`ResourceDisposition::NotAcquired`].
const ACQUIRED_AFTER_PUBLICATION: bool = true;

/// Exactly one terminal record per attempt, in one of the five shapes an attempt can actually have.
///
/// Both frozen originals settle with exactly one terminal record. This invocation schedules no
/// retry, and no retry identity is representable at all — see
/// [`AttemptOrdinal`](super::attempt_ordinal::AttemptOrdinal) — so there is no supersession field
/// here for one to be written under.
///
/// **Each shape carries exactly the facts its stage guarantees, and no field for facts it lacks.**
/// Widening those fields to `Option` would restore precisely the "complete record with missing
/// observations" shape the split exists to forbid.
///
/// The mapping is closed in both directions: [`AttemptStage`] has one shape each, and
/// [`FailureKind`](super::failure_kind::FailureKind) maps totally onto stages, so no failure kind can
/// inhabit two shapes. Every failure-bearing constructor validates through
/// [`AttemptStage::ensure_admits`] rather than enumerating kinds itself.
///
/// **A recorded series requires a complete bracket, structurally.** [`AttemptedOutcome`] — the only
/// type that can hold a [`CalibrationSeries`](super::calibration_series::CalibrationSeries) —
/// appears solely in [`Self::Attempted`], which requires non-optional [`AttemptProvenance`] and a
/// non-optional [`HostObservations`] pair.
///
/// **No `Debug`**, inherited from the retained samples.
#[derive(Clone, Serialize)]
pub(crate) enum CalibrationRecord {
    /// The slot never entered acquisition: the gate refused it, the gate could not be run, or the
    /// preceding attempt's release failed.
    NotRun {
        key: AttemptKey,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        schedule_seed: u64,
        reason: NotRunReason,
    },
    /// Acquisition began and failed before any instance existed.
    NotProvisioned {
        key: AttemptKey,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        schedule_seed: u64,
        partial_provision: PartialProvision,
        failure: AttemptFailure,
        release: ResourceDisposition,
    },
    /// The instance was published and fully identified, but the measurement window never opened.
    NotMeasured {
        key: AttemptKey,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        schedule_seed: u64,
        provenance: AttemptProvenance,
        failure: AttemptFailure,
        release: ResourceDisposition,
    },
    /// The `after` observation failed, so the bracket could not be closed. Retains the surviving
    /// `before` observation and whatever partial series the batch produced.
    Unbracketed {
        key: AttemptKey,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        schedule_seed: u64,
        provenance: AttemptProvenance,
        before: EnvironmentSample,
        failure: AttemptFailure,
        release: ResourceDisposition,
    },
    /// The measurement window opened and closed with both host observations. The only shape that can
    /// be `CalibrationRecorded`.
    Attempted {
        key: AttemptKey,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        schedule_seed: u64,
        provenance: AttemptProvenance,
        host: HostObservations,
        outcome: AttemptedOutcome,
        release: ResourceDisposition,
    },
}

impl CalibrationRecord {
    /// A slot that never entered acquisition.
    ///
    /// **Infallible, and deliberately typed that way.** The other four constructors return `Result`
    /// because they have something real to reject — a failure whose kind its stage cannot hold, or a
    /// disposition disagreeing with acquisition depth. This one has neither: there is no failure and
    /// no acquisition, and this module has no supersession link to validate, which is the reason the
    /// accepted screen's equivalent is fallible.
    ///
    /// That matters beyond tidiness. This constructor is the whole of slot settlement, and the frozen
    /// inventory promises one terminal record per predeclared attempt. A `Result` here would make
    /// [`settle_remaining`](super::calibration_driver) fallible, putting a `?` on precisely the path
    /// that exists to keep that promise — so a run stopping early could fail to record the slots it
    /// never reached, for a reason that cannot occur.
    pub(crate) fn not_run(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: u64,
        reason: NotRunReason,
    ) -> Self {
        Self::NotRun {
            key,
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            schedule_seed: seed,
            reason,
        }
    }

    /// An attempt that failed before any instance existed.
    pub(crate) fn not_provisioned(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: u64,
        partial_provision: PartialProvision,
        failure: AttemptFailure,
        release: ResourceDisposition,
    ) -> Result<Self> {
        AttemptStage::Unprovisioned.ensure_admits(&failure)?;
        release.ensure_matches_acquisition(partial_provision.depth().acquired_releasable())?;
        Ok(Self::NotProvisioned {
            key,
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            schedule_seed: seed,
            partial_provision,
            failure,
            release,
        })
    }

    /// An attempt whose instance existed but whose measurement window never opened.
    pub(crate) fn not_measured(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: u64,
        provenance: AttemptProvenance,
        failure: AttemptFailure,
        release: ResourceDisposition,
    ) -> Result<Self> {
        AttemptStage::Unmeasured.ensure_admits(&failure)?;
        release.ensure_matches_acquisition(ACQUIRED_AFTER_PUBLICATION)?;
        Ok(Self::NotMeasured {
            key,
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            schedule_seed: seed,
            provenance,
            failure,
            release,
        })
    }

    /// An attempt whose batch ran but whose bracket could not be closed.
    pub(crate) fn unbracketed(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: u64,
        provenance: AttemptProvenance,
        before: EnvironmentSample,
        failure: AttemptFailure,
        release: ResourceDisposition,
    ) -> Result<Self> {
        AttemptStage::Unbracketed.ensure_admits(&failure)?;
        release.ensure_matches_acquisition(ACQUIRED_AFTER_PUBLICATION)?;
        Ok(Self::Unbracketed {
            key,
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            schedule_seed: seed,
            provenance,
            before,
            failure,
            release,
        })
    }

    /// An attempt whose measurement window opened and closed with both host observations.
    ///
    /// A `CalibrationRecorded` outcome needs no stage check — it carries no failure — while a
    /// `Failed` one must be `Bracketed`, which is what stops a pre-measurement failure being dressed
    /// up as a measured result by supplying observations it never took.
    pub(crate) fn attempted(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: u64,
        provenance: AttemptProvenance,
        host: HostObservations,
        outcome: AttemptedOutcome,
        release: ResourceDisposition,
    ) -> Result<Self> {
        if let AttemptedOutcome::Failed { failure } = &outcome {
            AttemptStage::Bracketed.ensure_admits(failure)?;
        }
        release.ensure_matches_acquisition(ACQUIRED_AFTER_PUBLICATION)?;
        Ok(Self::Attempted {
            key,
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            schedule_seed: seed,
            provenance,
            host,
            outcome,
            release,
        })
    }

    /// The failing release's diagnostic, when this attempt's release did not succeed.
    pub(crate) fn release_failure_diagnostic(&self) -> Option<&DiagnosticArtifact> {
        match self {
            Self::NotRun { .. } => None,
            Self::NotProvisioned { release, .. }
            | Self::NotMeasured { release, .. }
            | Self::Unbracketed { release, .. }
            | Self::Attempted { release, .. } => release.failure_diagnostic(),
        }
    }
}
