//! What kind of failure ended an attempt.

use serde::Serialize;

use crate::control_registry_discovery_screen::attempt_stage::AttemptStage;
use crate::control_registry_discovery_screen::failure_phase::FailurePhase;

/// The failure classes this screen can produce, each naming one point in the driver's acquisition
/// and measurement path.
///
/// **A kind determines what evidence is possible**, which is why two failures are split in two.
/// `TimedSubscription` names the *measured* subscription failing to apply, so no sample exists;
/// `ValidationSubscription` names one of the three untimed validation subscriptions failing after
/// the timed apply already completed, so a raw sample does exist and must be retained. One kind
/// covering both would leave sample availability undeterminable from the ledger.
///
/// The `after`-observation failure is split for exactly the same reason. The observation is attempted
/// whether or not the timed subscription applied, so its failure has two genuinely different evidence
/// states behind it: `HostObservationAfterTimedFailure` follows a timed apply that never completed
/// and therefore has no sample, while `HostObservationAfter` follows one that did and must retain its
/// rejected nanoseconds. Collapsing them would force one kind to be both sample-bearing and not, which
/// is the biconditional in [`AttemptFailure`](super::attempt_failure::AttemptFailure) giving up.
///
/// A kind alone does not decide retry eligibility; see
/// [`AttemptFailure`](super::attempt_failure::AttemptFailure), which combines this kind's
/// [`FailurePhase`] with whether the first measured sample had begun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailureKind {
    /// Resolving the pinned distribution, staging the hash-verified module, starting the server, or
    /// publishing failed. No instance exists.
    Provision,
    /// The client could not connect to the published instance.
    Connect,
    /// A seeding reducer refused or errored.
    Reducer,
    /// The pre-measurement host observation could not be taken, so the window never opened.
    HostObservationBefore,
    /// The timed subscription failed to apply, and the `after` observation that followed it
    /// succeeded. No sample was produced, but the host bracket is complete.
    TimedSubscription,
    /// The timed subscription failed to apply *and* the `after` observation attempted immediately
    /// afterwards also failed. No sample exists and no bracket can be closed; the record carries one
    /// chained diagnostic retaining both failures.
    ///
    /// Without this kind the pair would be unrecordable: `TimedSubscription` demands the complete
    /// host pair its shape carries, and `HostObservationAfter` demands the raw sample a failed apply
    /// never produced.
    HostObservationAfterTimedFailure,
    /// The timed apply completed and the post-measurement host observation could not be taken, so
    /// the bracket cannot be closed. The completed apply's raw duration survives as non-evidence.
    HostObservationAfter,
    /// One of the three untimed validation subscriptions failed to apply after the timed interval,
    /// so the four caches could not be read.
    ValidationSubscription,
    /// The apply sample was nonpositive and could not seal.
    Sample,
    /// The four-way composition check did not match the frozen expectation.
    Semantics,
}

impl FailureKind {
    /// Every kind, for exhaustive checks.
    pub(crate) const ALL: [FailureKind; 10] = [
        FailureKind::Provision,
        FailureKind::Connect,
        FailureKind::Reducer,
        FailureKind::HostObservationBefore,
        FailureKind::TimedSubscription,
        FailureKind::HostObservationAfterTimedFailure,
        FailureKind::HostObservationAfter,
        FailureKind::ValidationSubscription,
        FailureKind::Sample,
        FailureKind::Semantics,
    ];

    /// The one record shape this kind's evidence supports.
    ///
    /// Total, and the sole mapping from kind to shape — every record constructor validates through
    /// it rather than enumerating kinds itself, so a kind cannot inhabit two shapes.
    pub(crate) fn stage(self) -> AttemptStage {
        match self {
            Self::Provision => AttemptStage::Unprovisioned,
            Self::Connect | Self::Reducer | Self::HostObservationBefore => AttemptStage::Unmeasured,
            Self::HostObservationAfterTimedFailure | Self::HostObservationAfter => {
                AttemptStage::Unbracketed
            }
            Self::TimedSubscription
            | Self::ValidationSubscription
            | Self::Sample
            | Self::Semantics => AttemptStage::Bracketed,
        }
    }

    /// Whether this kind is an infrastructure failure, a result produced by the system under test,
    /// or a failure of the harness's own host observation.
    ///
    /// The spec's retry rule turns on this: only a *prospective infrastructure* failure before the
    /// first measured sample is retryable. A seeding reducer refusal is an application error even
    /// though it is pre-sample, and a `/proc` read failure is neither infrastructure under test nor
    /// an application result — it is the harness failing to record the method's required context,
    /// which the spec makes non-retryable in its own right.
    pub(crate) fn phase(self) -> FailurePhase {
        match self {
            Self::Provision | Self::Connect => FailurePhase::Infrastructure,
            Self::HostObservationBefore | Self::HostObservationAfter => {
                FailurePhase::HarnessObservation
            }
            // `HostObservationAfterTimedFailure` sits here, with `TimedSubscription`, rather than
            // with the two host-observation kinds it is named alongside. Its causally primary
            // failure *is* the timed subscription failing to apply; the observation failure that
            // follows only decides which record shape can hold it. Classifying it by the later
            // failure would let one measurement outcome change phase according to whether a
            // subsequent `/proc` read happened to succeed, which is a fact about the harness, not
            // about the measurement.
            Self::Reducer
            | Self::TimedSubscription
            | Self::HostObservationAfterTimedFailure
            | Self::ValidationSubscription
            | Self::Sample
            | Self::Semantics => FailurePhase::ApplicationOrSemantic,
        }
    }

    /// Whether the first measured sample had begun when this kind can strike.
    ///
    /// Derived from [`Self::stage`] rather than matched separately, so the two cannot drift: the
    /// measurement window is exactly what `Bracketed` and `Unbracketed` mean, and every earlier
    /// stage precedes the timed subscription's issue. This is what forces every pre-measurement
    /// kind to `BeforeFirstSample` without the driver being trusted to say so.
    pub(crate) fn can_follow_first_sample(self) -> bool {
        matches!(
            self.stage(),
            AttemptStage::Bracketed | AttemptStage::Unbracketed
        )
    }

    /// Whether a failure of this kind necessarily has a completed timed apply behind it.
    ///
    /// True exactly for the four kinds that can only be reached *after* the timed subscription
    /// applied, so each must retain its raw nanoseconds. Enforced as a biconditional against the
    /// partial evidence, which is what makes raw-sample retention a structural property rather than
    /// a habit: a `Semantics` mismatch reporting no sample, or a `Connect` failure reporting one,
    /// are both rejected.
    ///
    /// The two exclusions inside the measurement window are the instructive ones.
    /// `TimedSubscription` names that very apply failing, so it is `Bracketed` yet has no sample;
    /// `HostObservationAfterTimedFailure` is `Unbracketed` and has none for the same reason. Only
    /// `HostObservationAfter` — the *other* after-observation failure, the one reached past a
    /// completed apply — carries a sample.
    pub(crate) fn requires_sample(self) -> bool {
        match self {
            Self::Provision
            | Self::Connect
            | Self::Reducer
            | Self::HostObservationBefore
            | Self::TimedSubscription
            | Self::HostObservationAfterTimedFailure => false,
            Self::HostObservationAfter
            | Self::ValidationSubscription
            | Self::Sample
            | Self::Semantics => true,
        }
    }

    /// Whether a failure of this kind can have read the four caches.
    ///
    /// The `after` observation is taken immediately after the timed adapter returns and *before* any
    /// validation subscription or cache read, so both after-observation kinds and
    /// `ValidationSubscription` all strike while the caches are still unread. Only `Sample` and
    /// `Semantics` are reached past the reads.
    pub(crate) fn can_observe_composition(self) -> bool {
        matches!(self, Self::Sample | Self::Semantics)
    }

    /// Whether a failure of this kind is meaningless without an observed composition.
    ///
    /// True only for `Semantics`, which *is* an observed-versus-expected mismatch: a semantic
    /// failure reporting that nothing was observed would be asserting a mismatch it never saw.
    pub(crate) fn requires_observed_composition(self) -> bool {
        matches!(self, Self::Semantics)
    }
}
