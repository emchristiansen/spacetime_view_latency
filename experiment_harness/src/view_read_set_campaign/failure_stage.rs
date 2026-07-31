//! How far through its lifecycle a failed attempt had actually got.

use serde::Serialize;

use crate::view_read_set_campaign::measured_sample_boundary::MeasuredSampleBoundary;

/// The two lifecycle boundaries a failed attempt is judged against, as one closed sum.
///
/// A failed attempt has to answer two questions that no cause vocabulary can:
///
/// - **Had it published?** This decides whether a `Provisioned` provenance line must exist for it.
///   [`FailureKind`](super::failure_kind::FailureKind) cannot answer it — a server-start timeout is
///   classified [`Timeout`](super::failure_kind::FailureKind::Timeout) and happens before publish,
///   while a reducer timeout is the same variant after it — and neither can
///   [`InfrastructurePhase`](super::infrastructure_phase::InfrastructurePhase), which only exists
///   inside one of the five causes.
/// - **Had it taken a measured sample?** This is the timing term of the retry rule.
///
/// **Why one field and not two.** The two answers are not independent: an attempt that never
/// published cannot have measured anything. Two separate fields would make
/// `(BeforePublish, AfterFirst)` a well-typed value describing an attempt that measured a server it
/// never created, and reconciliation would have to reject at runtime what should not exist. Three
/// variants express exactly the three reachable states, and both derived facts come out of a total
/// match rather than out of a consistency check.
///
/// **It is stated by the driver, not inferred.** Nothing in the retained evidence recovers either
/// answer: an empty channel prefix is identical whether measurement had begun or not, and the spec
/// freezes the order of the four channels but not where composition validation sits among them. A
/// rule that guessed here could reference a measured outcome by accident, which is precisely what
/// the contract forbids. So the driver — the only thing that knows — records it, and it is
/// serialized with the terminal record so a reader audits the retry and provenance decisions rather
/// than re-deriving them.
///
/// [`MeasuredSampleBoundary`] remains the type the retry rule reads; it is now *derived* from this
/// one by [`Self::measured_sample_boundary`] rather than stored beside it, so the two cannot
/// disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailureStage {
    /// The attempt failed before its module was published. It has no provisioned instance, so no
    /// `Provisioned` line exists for it, and by construction it took no measured sample.
    BeforePublish,
    /// The attempt published, and failed before its first measured sample. It has a provisioned
    /// instance. This is the retry-eligible stage, for an infrastructure cause.
    AfterPublishBeforeFirstSample,
    /// The attempt published and had taken at least one measured sample. It has a provisioned
    /// instance, and a post-attempt environment reading is required for it — it is a measured
    /// attempt. Never retry-eligible, whatever the cause.
    AfterFirstSample,
}

impl FailureStage {
    /// Whether the attempt got far enough to have a published instance, and therefore must have a
    /// `Provisioned` provenance line in the ledger.
    ///
    /// Total over the three stages, so the provisioning rule for a failed attempt is decided by the
    /// type rather than reconstructed from a cause and a phase.
    pub(crate) fn published(self) -> bool {
        match self {
            Self::BeforePublish => false,
            Self::AfterPublishBeforeFirstSample | Self::AfterFirstSample => true,
        }
    }

    /// Which side of its first measured sample the attempt failed on — the retry rule's timing term.
    pub(crate) fn measured_sample_boundary(self) -> MeasuredSampleBoundary {
        match self {
            Self::BeforePublish | Self::AfterPublishBeforeFirstSample => {
                MeasuredSampleBoundary::BeforeFirst
            }
            Self::AfterFirstSample => MeasuredSampleBoundary::AfterFirst,
        }
    }
}
