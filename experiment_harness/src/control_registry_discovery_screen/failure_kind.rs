//! What kind of failure ended an attempt.

use serde::Serialize;

use crate::control_registry_discovery_screen::failure_phase::FailurePhase;

/// The failure classes this screen can produce, copied from the Pilot's vocabulary.
///
/// Every variant is reachable here — provisioning a pinned instance, connecting, the seeding
/// reducers, the timed subscription, the apply sample itself, and the four-way composition check —
/// so the enum carries only variants this module can actually produce.
///
/// A kind alone does not decide retry eligibility; see
/// [`AttemptFailure`](super::attempt_failure::AttemptFailure), which combines it with whether the
/// first measured sample had begun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailureKind {
    /// Provisioning the pinned distribution, server, or data directory failed.
    Provision,
    /// The client could not connect to the provisioned server.
    Connect,
    /// A seeding reducer refused or errored.
    Reducer,
    /// The timed subscription failed to apply.
    Subscription,
    /// The apply sample was missing or nonpositive.
    Sample,
    /// The four-way composition check did not match the frozen expectation.
    Semantics,
}

impl FailureKind {
    /// Whether this kind is an infrastructure failure or an application/semantic one.
    ///
    /// The spec's retry rule turns on this split: prospective *infrastructure* failure before the
    /// first measured sample is retryable, while semantic failures, security failures, application
    /// errors, timeouts, and nonpositive statistics never are. A seeding reducer refusal is an
    /// application error, not infrastructure, even though it happens before any sample.
    pub(crate) fn phase(self) -> FailurePhase {
        match self {
            Self::Provision | Self::Connect => FailurePhase::Infrastructure,
            Self::Reducer | Self::Subscription | Self::Sample | Self::Semantics => {
                FailurePhase::ApplicationOrSemantic
            }
        }
    }

    /// Whether this kind can still be observed once the first measured sample has begun.
    ///
    /// False for provisioning, connection, and seeding because all three strictly precede the timed
    /// subscription in this driver — every seeding reducer completes before the subscription begins
    /// — so a record claiming any of them followed the first sample describes an order of events
    /// that cannot occur. Enforced when a failure is constructed, so that shape is rejected rather
    /// than written.
    ///
    /// `Reducer` is the case that shows why this is a separate question from [`Self::phase`]: it is
    /// pre-sample like the other two, yet never retryable, because it is an application error rather
    /// than an infrastructure one. Collapsing the two dimensions into one would make it retryable.
    pub(crate) fn can_follow_first_sample(self) -> bool {
        match self {
            Self::Provision | Self::Connect | Self::Reducer => false,
            Self::Subscription | Self::Sample | Self::Semantics => true,
        }
    }

    /// Whether a failure of this kind can have read its target's cardinality.
    ///
    /// Reading a cardinality means the subscription applied, so only the kinds that strike *after*
    /// application qualify. `Subscription` is the instructive exclusion: it names the subscription
    /// failing to apply at all, so a `Subscription` failure carrying an observed row count would be
    /// claiming to have read a cache that never materialized.
    pub(crate) fn can_observe_composition(self) -> bool {
        match self {
            Self::Provision | Self::Connect | Self::Reducer | Self::Subscription => false,
            Self::Sample | Self::Semantics => true,
        }
    }

    /// Whether a failure of this kind is meaningless without an observed cardinality.
    ///
    /// True only for `Semantics`, which *is* an observed-versus-expected composition mismatch: a
    /// semantic failure reporting that nothing was observed would be asserting a mismatch it never
    /// saw.
    pub(crate) fn requires_observed_composition(self) -> bool {
        matches!(self, Self::Semantics)
    }
}
