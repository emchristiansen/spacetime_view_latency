//! One attempt's durable ledger line.

use anyhow::Result;
use serde::Serialize;

use crate::control_registry_discovery_screen::attempt_failure::AttemptFailure;
use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::attempt_stage::AttemptStage;
use crate::control_registry_discovery_screen::attempted_outcome::AttemptedOutcome;
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::host_observations::HostObservations;
use crate::control_registry_discovery_screen::method_facts::MethodFacts;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::partial_provision::PartialProvision;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::retry_ordinal::RetryOrdinal;
use crate::control_registry_discovery_screen::screen_composition::ScreenComposition;
use crate::control_registry_discovery_screen::supersession::Supersession;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// Whether a record shape past publication owned a releasable resource. Always, and unconditionally:
/// reaching publication means the staged module tempfile and the started server with its fresh data
/// directory were both the driver's to release, so every post-publication shape rejects
/// [`ResourceDisposition::NotAcquired`]. Named rather than written as a bare `true` at three call
/// sites, so the claim reads as the fact it is.
const ACQUIRED_AFTER_PUBLICATION: bool = true;

/// Exactly one terminal record per attempt, in one of the five shapes an attempt can actually have.
///
/// Every one of the sixteen frozen original logical slots settles with exactly one terminal record
/// in a run. This screen schedules no retry, so it writes exactly those sixteen; the model can
/// nonetheless represent a retry identity, which would append its own record under a distinct
/// identity carrying an explicit supersession link and never replace the original's.
///
/// **Each shape carries exactly the facts its stage guarantees, and no field for facts it lacks.**
/// The Phase 1 two-shape model required full post-publication provenance on every failed attempt,
/// which no failure before publication can supply; widening those fields to `Option` would have
/// restored precisely the "complete record with missing observations" shape the split exists to
/// forbid. Five shapes with total fields say the same thing without lying.
///
/// The mapping is closed in both directions: [`AttemptStage`] has one shape each, and
/// [`FailureKind`](super::failure_kind::FailureKind) maps totally onto stages, so no failure kind
/// can inhabit two shapes. Every failure-bearing constructor validates through
/// [`AttemptStage::ensure_admits`] rather than enumerating kinds itself.
///
/// **Evidence requires a complete bracket, structurally.** [`AttemptedOutcome`] — the only type that
/// can hold [`ColdApplyEvidence`](super::cold_apply_evidence::ColdApplyEvidence) — appears solely in
/// [`Self::Attempted`], which requires non-optional [`AttemptProvenance`] and a non-optional
/// [`HostObservations`] pair. [`Self::Unbracketed`] carries an [`AttemptFailure`] instead, so a
/// measurement whose bracket could not be closed cannot be recorded as complete at all.
///
/// Identity, composition, frozen method facts, pinned artifact identity, and seed appear on *every*
/// shape: a reader holding only the ledger must see what a slot was going to measure, under what
/// method, against which pinned artifacts, even when it never ran. Composition, method, and
/// supersession are derived from the identity inside these constructors rather than accepted, so a
/// record cannot describe a composition its own identity contradicts.
///
/// **No `Debug`**, inherited from the retained rejected samples — see
/// [`RejectedApplyNanos`](super::rejected_apply_nanos::RejectedApplyNanos).
#[derive(Clone, Serialize)]
pub(crate) enum ScreenRecord {
    /// The slot never entered acquisition: the gate refused it, the gate could not be run, or a
    /// preceding attempt's release failed. It measured nothing and acquired nothing, so it carries
    /// neither provenance nor a resource disposition.
    NotRun {
        key: AttemptKey,
        composition: ScreenComposition,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        supersession: Supersession,
        schedule_seed: u64,
        reason: NotRunReason,
    },
    /// Acquisition began and failed before any instance existed. Carries the greatest verified
    /// provisioning prefix it reached.
    NotProvisioned {
        key: AttemptKey,
        composition: ScreenComposition,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        supersession: Supersession,
        schedule_seed: u64,
        partial_provision: PartialProvision,
        failure: AttemptFailure,
        release: ResourceDisposition,
    },
    /// The instance was published and fully identified, but the measurement window never opened, so
    /// there is no host observation to record.
    NotMeasured {
        key: AttemptKey,
        composition: ScreenComposition,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        supersession: Supersession,
        schedule_seed: u64,
        provenance: AttemptProvenance,
        failure: AttemptFailure,
        release: ResourceDisposition,
    },
    /// The `after` observation failed, so the bracket could not be closed. Retains the surviving
    /// `before` observation, and — where the timed apply had completed first — the rejected raw
    /// sample carried by the failure's partial evidence. Where it had not, the failure is
    /// `HostObservationAfterTimedFailure` and carries one chained diagnostic instead.
    Unbracketed {
        key: AttemptKey,
        composition: ScreenComposition,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        supersession: Supersession,
        schedule_seed: u64,
        provenance: AttemptProvenance,
        before: EnvironmentSample,
        failure: AttemptFailure,
        release: ResourceDisposition,
    },
    /// The measurement window opened and closed with both host observations. The only shape that
    /// can be `Complete`.
    Attempted {
        key: AttemptKey,
        composition: ScreenComposition,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        supersession: Supersession,
        schedule_seed: u64,
        provenance: AttemptProvenance,
        host: HostObservations,
        outcome: AttemptedOutcome,
        release: ResourceDisposition,
    },
}

impl ScreenRecord {
    /// A slot that never entered acquisition.
    ///
    /// `superseded` is the ordinal this identity replaces, or `None` for an original. This screen's
    /// sixteen frozen identities and its remaining-slot settlement are all originals and always pass
    /// `None`; the parameter exists because the record model must be able to *represent* a retry
    /// identity, not because this invocation mints one.
    pub(crate) fn not_run(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: ScheduleSeed,
        superseded: Option<RetryOrdinal>,
        reason: NotRunReason,
    ) -> Result<Self> {
        Ok(Self::NotRun {
            key,
            composition: ScreenComposition::of(key),
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            supersession: Supersession::of(key.retry(), superseded)?,
            schedule_seed: seed.get(),
            reason,
        })
    }

    /// An attempt that failed before any instance existed.
    pub(crate) fn not_provisioned(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: ScheduleSeed,
        superseded: Option<RetryOrdinal>,
        partial_provision: PartialProvision,
        failure: AttemptFailure,
        release: ResourceDisposition,
    ) -> Result<Self> {
        AttemptStage::Unprovisioned.ensure_admits(&failure)?;
        release.ensure_matches_acquisition(partial_provision.depth().acquired_releasable())?;
        Ok(Self::NotProvisioned {
            key,
            composition: ScreenComposition::of(key),
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            supersession: Supersession::of(key.retry(), superseded)?,
            schedule_seed: seed.get(),
            partial_provision,
            failure,
            release,
        })
    }

    /// An attempt whose instance existed but whose measurement window never opened.
    pub(crate) fn not_measured(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: ScheduleSeed,
        superseded: Option<RetryOrdinal>,
        provenance: AttemptProvenance,
        failure: AttemptFailure,
        release: ResourceDisposition,
    ) -> Result<Self> {
        AttemptStage::Unmeasured.ensure_admits(&failure)?;
        release.ensure_matches_acquisition(ACQUIRED_AFTER_PUBLICATION)?;
        Ok(Self::NotMeasured {
            key,
            composition: ScreenComposition::of(key),
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            supersession: Supersession::of(key.retry(), superseded)?,
            schedule_seed: seed.get(),
            provenance,
            failure,
            release,
        })
    }

    /// An attempt whose timed apply completed but whose bracket could not be closed.
    pub(crate) fn unbracketed(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: ScheduleSeed,
        superseded: Option<RetryOrdinal>,
        provenance: AttemptProvenance,
        before: EnvironmentSample,
        failure: AttemptFailure,
        release: ResourceDisposition,
    ) -> Result<Self> {
        AttemptStage::Unbracketed.ensure_admits(&failure)?;
        release.ensure_matches_acquisition(ACQUIRED_AFTER_PUBLICATION)?;
        Ok(Self::Unbracketed {
            key,
            composition: ScreenComposition::of(key),
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            supersession: Supersession::of(key.retry(), superseded)?,
            schedule_seed: seed.get(),
            provenance,
            before,
            failure,
            release,
        })
    }

    /// An attempt whose measurement window opened and closed with both host observations.
    ///
    /// A `Complete` outcome needs no stage check — it carries no failure — while a `Failed` one must
    /// be `Bracketed`, which is what stops a pre-measurement failure being dressed up as a measured
    /// result by supplying observations it never took.
    pub(crate) fn attempted(
        key: AttemptKey,
        pinned: &PinnedArtifactIdentity,
        seed: ScheduleSeed,
        superseded: Option<RetryOrdinal>,
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
            composition: ScreenComposition::of(key),
            method: MethodFacts::frozen(),
            pinned: pinned.clone(),
            supersession: Supersession::of(key.retry(), superseded)?,
            schedule_seed: seed.get(),
            provenance,
            host,
            outcome,
            release,
        })
    }

    /// The failing release's diagnostic, when this attempt's release did not succeed.
    ///
    /// The driver reads this off the record it has just appended, so its decision to mark every
    /// remaining frozen slot `NotRun(PriorAttemptReleaseFailed)` and the durable evidence for that
    /// decision come from the same value.
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

#[cfg(test)]
mod tests;
