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
    /// The batch ran to the frozen count and the post-measurement host observation could not be
    /// taken, so the bracket cannot be closed. The complete series survives as non-evidence.
    HostObservationAfter,
    /// The batch ran to its end and the series could not seal — a nonpositive sample, or a length
    /// that is not exactly the frozen complete count.
    Sample,
    /// The composition check did not match the frozen expectation.
    Semantics,
}

impl FailureKind {
    /// Every kind, for exhaustive checks.
    pub(crate) const ALL: [FailureKind; 10] = [
        FailureKind::Provision,
        FailureKind::Connect,
        FailureKind::Reducer,
        FailureKind::Subscription,
        FailureKind::HostObservationBefore,
        FailureKind::PacedBatch,
        FailureKind::HostObservationAfterBatchFailure,
        FailureKind::HostObservationAfter,
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
            Self::PacedBatch | Self::Sample | Self::Semantics => AttemptStage::Bracketed,
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

    /// Whether a failure of this kind necessarily has a **complete** batch behind it, so a retained
    /// series is mandatory.
    ///
    /// True exactly for the three kinds reachable only after the batch ran to its end. Note that
    /// "ran to its end" is not "reached the frozen count": [`Self::Sample`] is precisely the kind for
    /// a batch that terminated normally and still produced a length other than the frozen one, and it
    /// retains those samples.
    /// Enforced together with [`Self::permits_series`] as a pair of implications rather than the
    /// biconditional the discovery screen can afford.
    ///
    /// **Why not a biconditional.** E3's measured unit is one cold apply: it either completed or it
    /// did not, so "kind requires a sample" and "evidence carries one" coincide exactly. A paced
    /// batch is up to a thousand appends issued one at a time, so a batch that stops at 400
    /// genuinely holds four hundred ordered samples — informative about precisely the stationarity
    /// and lag questions this pilot exists to answer. Forcing the E3 biconditional here would make
    /// that partial series unrecordable, and discarding it would throw away most of what a failed
    /// attempt learned.
    pub(crate) fn requires_series(self) -> bool {
        matches!(
            self,
            Self::HostObservationAfter | Self::Sample | Self::Semantics
        )
    }

    /// Whether a failure of this kind *may* carry a retained series.
    ///
    /// Every kind that requires one, plus the two that follow a batch which began and stopped early.
    /// A kind that strictly precedes the first append carries none, and a partial evidence value
    /// claiming otherwise is rejected — a `Connect` failure reporting four hundred samples is
    /// inventing them.
    pub(crate) fn permits_series(self) -> bool {
        self.requires_series()
            || matches!(
                self,
                Self::PacedBatch | Self::HostObservationAfterBatchFailure
            )
    }

    /// Whether a failure of this kind can have read the two caches.
    ///
    /// The `after` observation is taken immediately after the paced batch returns and *before* the
    /// untimed witness subscription or any cache read, so both after-observation kinds strike while
    /// the caches are still unread. `PacedBatch` likewise stops before them. Only `Sample` and
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

#[cfg(test)]
mod tests;
