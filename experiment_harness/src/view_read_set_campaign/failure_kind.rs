//! What kind of failure terminated an attempt after it launched.

use serde::Serialize;

use crate::view_read_set_campaign::infrastructure_phase::InfrastructurePhase;

/// The classified cause that terminated a launched attempt, partitioned by what the retry rule does
/// with it.
///
/// **Why these classes and not lifecycle stages.** The spec's retry rule reads on cause: "an attempt
/// is retryable only for prospective environment-gate invalidation or infrastructure failure
/// occurring before its first measured sample; semantic failures, security failures, application
/// errors, timeouts, and nonpositive statistics are never retryable". A vocabulary organised by
/// *where* a failure happened — connect, subscription, sample — cannot answer that, because a
/// connect failure may be a dead socket or a timeout and those fall on opposite sides of the rule.
/// So the variants are the rule's own categories, and the lifecycle position rides along inside
/// [`Self::Infrastructure`] as [`InfrastructurePhase`], where it is diagnostic rather than decisive.
///
/// **The gate's clause is not here.** The rule's other retryable branch is prospective
/// environment-gate invalidation, which is not a failure of a run at all — nothing had launched. It
/// is [`AttemptOutcome::PreflightRejected`](super::attempt_outcome::AttemptOutcome::PreflightRejected),
/// a sibling of the variant this enum lives inside, so it can carry its gate readings and *no*
/// evidence field.
///
/// **What each class settles about retry.** [`Self::Infrastructure`] is *conditionally* eligible: it
/// qualifies only together with the first-measured-sample boundary, which this enum does not record,
/// and never merely by being infrastructure. Every other variant is categorically **ineligible**,
/// however early it occurred, because each is a measured outcome and "no retry criterion may
/// reference a measured or post-attempt outcome".
///
/// The eventual retry decision is therefore a total function over the terminal outcome *plus* the
/// first-sample boundary. This type carries the whole of the cause term, so that function can be
/// written without re-deriving anything from a diagnostic string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailureKind {
    /// The harness, distribution, server process, or transport failed — not the candidate. The phase
    /// says only where it happened, never whether it preceded the first measured sample.
    Infrastructure(InfrastructurePhase),
    /// The module or one of its reducers returned an error. A measured outcome: never retryable.
    Application,
    /// An operation exceeded its bound. Named separately from [`Self::Infrastructure`] precisely
    /// because a timeout that *looks* like an infrastructure fault is still categorically
    /// nonretryable — folding it in would let the rule be relaxed by reclassification.
    Timeout,
    /// A channel reduced to a statistic that was not strictly positive, refused at
    /// [`CellStatistic::validated`](super::cell_statistic::CellStatistic::validated). Never
    /// retryable, and never counted as flat.
    NonpositiveStatistic,
    /// A semantic or authorization gate over the observed result set failed — for the Arm, the
    /// sender-scoped view returning any of the foreign slice. Never retryable: a security failure is
    /// the candidate's answer, not an accident of the run.
    SemanticsOrSecurity,
}
