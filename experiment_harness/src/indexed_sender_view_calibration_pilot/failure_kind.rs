//! What kind of failure ended an attempt.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::attempt_stage::AttemptStage;
use crate::indexed_sender_view_calibration_pilot::failure_phase::FailurePhase;

/// The failure classes this pilot can produce, each naming one point in the driver's acquisition and
/// measurement path.
///
/// **A kind determines what evidence is possible.** Two pairs are split for exactly that reason.
/// `PacedBatch` names the measured batch itself stopping early, so a *partial* series may exist;
/// `Sample` names a batch that ran to its end and still could not seal — a nonpositive sample, or a
/// length that is not the frozen complete count. The `after`-observation failure splits the same
/// way: the observation is attempted whether or not the batch completed, so
/// `HostObservationAfterBatchFailure` follows a batch that stopped early and
/// `HostObservationAfter` one that ran to the end.
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
    /// A seeding reducer refused or errored while establishing the unrelated population or the
    /// subscriber's own slice.
    Reducer,
    /// The measured arm's subscription failed to apply.
    ///
    /// Pre-measurement, and that is the whole reason it is its own kind rather than folded into the
    /// batch. E2 requires the subscription to be live *before* the first append, because the
    /// endpoint is the appended row becoming visible in a cache that must already exist. A
    /// subscription that never applied leaves nothing to measure into.
    Subscription,
    /// The pre-measurement host observation could not be taken, so the window never opened.
    HostObservationBefore,
    /// The paced batch stopped before recording the frozen count, and the `after` observation that
    /// followed it succeeded. Whatever samples it did record are retained in order.
    PacedBatch,
    /// The paced batch stopped early *and* the `after` observation attempted immediately afterwards
    /// also failed. The record carries one chained diagnostic retaining both failures, alongside
    /// whatever partial series existed.
    HostObservationAfterBatchFailure,
    /// The paced batch returned and the post-measurement host observation could not be taken, so the
    /// bracket cannot be closed. The returned series survives as non-evidence.
    ///
    /// Says "returned" rather than "reached the frozen count" deliberately. A batch that stops early
    /// and is *followed* by a failing observation is
    /// [`HostObservationAfterBatchFailure`](Self::HostObservationAfterBatchFailure), so in practice
    /// this kind follows a complete batch — but nothing here enforces the length, and
    /// [`CalibrationSeries::recorded`](super::calibration_series::CalibrationSeries::recorded) is the
    /// sole authority on completeness. Claiming a complete series in a doc that does not check one
    /// would be the overclaim this vocabulary exists to avoid.
    HostObservationAfter,
    /// The untimed composition-witness subscription failed to apply, so the caches could not be
    /// read and the composition could not be checked.
    ///
    /// Named for this module's own vocabulary rather than copied verbatim from the accepted screen's
    /// `ValidationSubscription`: there is exactly one untimed subscription here, and every other type
    /// in this module calls the relation it materializes the
    /// [`CompositionWitness`](super::calibration_target::CalibrationTarget::CompositionWitness).
    ///
    /// Reached only after a completed paced batch and a closed host bracket, and strictly *before*
    /// any cache read — so it carries the batch's raw series and can never claim a composition it
    /// never observed. Its mapping is the accepted screen's exactly.
    WitnessSubscription,
    /// The batch ran to its end and the series could not seal — a nonpositive sample, or a length
    /// that is not exactly the frozen complete count.
    Sample,
    /// The composition check did not match the frozen expectation.
    Semantics,
}

impl FailureKind {
    /// Every kind, for exhaustive checks.
    pub(crate) const ALL: [FailureKind; 11] = [
        FailureKind::Provision,
        FailureKind::Connect,
        FailureKind::Reducer,
        FailureKind::Subscription,
        FailureKind::HostObservationBefore,
        FailureKind::PacedBatch,
        FailureKind::HostObservationAfterBatchFailure,
        FailureKind::HostObservationAfter,
        FailureKind::WitnessSubscription,
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
            Self::Connect | Self::Reducer | Self::Subscription | Self::HostObservationBefore => {
                AttemptStage::Unmeasured
            }
            Self::HostObservationAfterBatchFailure | Self::HostObservationAfter => {
                AttemptStage::Unbracketed
            }
            Self::PacedBatch
            | Self::WitnessSubscription
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
    /// an application result.
    pub(crate) fn phase(self) -> FailurePhase {
        match self {
            Self::Provision | Self::Connect => FailurePhase::Infrastructure,
            Self::HostObservationBefore | Self::HostObservationAfter => {
                FailurePhase::HarnessObservation
            }
            // `HostObservationAfterBatchFailure` sits with `PacedBatch` rather than with the two
            // host-observation kinds it is named alongside, for the reason the discovery screen
            // gives: its causally primary failure *is* the batch stopping early, and the observation
            // failure that follows only decides which record shape can hold it. Classifying it by
            // the later failure would let one measurement outcome change phase according to whether
            // a subsequent `/proc` read happened to succeed.
            Self::Reducer
            | Self::Subscription
            | Self::PacedBatch
            | Self::HostObservationAfterBatchFailure
            | Self::WitnessSubscription
            | Self::Sample
            | Self::Semantics => FailurePhase::ApplicationOrSemantic,
        }
    }

    /// Whether the first measured sample had begun when this kind can strike.
    ///
    /// Derived from [`Self::stage`] rather than matched separately, so the two cannot drift: the
    /// measurement window is exactly what `Bracketed` and `Unbracketed` mean, and every earlier
    /// stage precedes the first append's issue.
    pub(crate) fn can_follow_first_sample(self) -> bool {
        matches!(
            self.stage(),
            AttemptStage::Bracketed | AttemptStage::Unbracketed
        )
    }

    /// Whether a failure of this kind must carry a retained series — **a biconditional with
    /// [`Self::can_follow_first_sample`], not a pair of implications.**
    ///
    /// A kind that strictly precedes the first append carries no series; a kind that can follow it
    /// carries exactly one. There is no middle. A `Connect` failure reporting four hundred samples is
    /// inventing them, and a `PacedBatch` failure reporting *no* series is denying that the batch it
    /// is named for ever began.
    ///
    /// **An earlier draft weakened this to two implications**, arguing that a paced batch is many
    /// samples where E3's cold apply is one, so a batch failing partway holds a genuine partial series
    /// that a biconditional would make unrecordable. The premise was right and the conclusion was
    /// wrong: partiality is carried by the series being *short or empty*, never by its being absent.
    /// [`RejectedSeries::of`](super::rejected_series::RejectedSeries::of) accepts an empty vector
    /// precisely so a batch that failed on its first append records "measured, and got nothing" —
    /// a different fact from "never reached the batch", and exactly the one the weakened form let a
    /// record blur. Requiring the container, empty or not, keeps both facts and fabricates no sample.
    ///
    /// "Can follow the first append" is not "reached the frozen count": [`Self::Sample`] is precisely
    /// the kind for a batch that returned with a length other than the frozen one, and it retains
    /// those samples.
    ///
    /// **Stated as an exhaustive match, not derived from [`Self::can_follow_first_sample`].** The two
    /// are equal today, and the taxonomy test asserts that equality — but they are different claims:
    /// one is about where a failure sits relative to the measurement window, the other about what
    /// evidence a record must carry. Deriving this from that would let a future change to
    /// [`Self::stage`] silently rewrite the evidence rule; writing it out means adding a kind forces
    /// a deliberate answer here, and moving a kind's stage breaks the test loudly instead.
    pub(crate) fn requires_series(self) -> bool {
        match self {
            Self::Provision
            | Self::Connect
            | Self::Reducer
            | Self::Subscription
            | Self::HostObservationBefore => false,
            Self::PacedBatch
            | Self::HostObservationAfterBatchFailure
            | Self::HostObservationAfter
            | Self::WitnessSubscription
            | Self::Sample
            | Self::Semantics => true,
        }
    }

    /// Whether a failure of this kind can have read the two caches.
    ///
    /// **A reachability fact, no longer the mismatch rule.** Carrying a mismatch is now permitted
    /// exactly for [`Self::requires_observed_composition`] — that is, for `Semantics` alone — because
    /// a mismatch filed under `Sample` would hide a semantic divergence behind a label naming a
    /// different fault. This predicate remains as the weaker statement about *when the reads
    /// happened*, which the taxonomy test pins against that rule.
    ///
    /// The `after` observation is taken immediately after the paced batch returns and *before* the
    /// untimed witness subscription or any cache read, so both after-observation kinds strike while
    /// the caches are still unread. `PacedBatch` likewise stops before them, and
    /// `WitnessSubscription` *is* the subscription that would have made the reads possible failing —
    /// so it too precedes them. Only `Sample` and `Semantics` are reached past the reads.
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

#[cfg(test)]
mod tests;
