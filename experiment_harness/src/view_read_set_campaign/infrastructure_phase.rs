//! Where in an attempt's lifecycle an infrastructure failure occurred.

use serde::Serialize;

/// The lifecycle boundary an infrastructure failure was hit at.
///
/// Diagnostic, not decisive. It is carried inside
/// [`FailureKind::Infrastructure`](super::failure_kind::FailureKind::Infrastructure) so a reader can
/// tell a server that never started from a subscription that dropped, without parsing the retained
/// diagnostic text.
///
/// It is deliberately *not* the retry rule's timing term. The rule turns on whether the failure
/// preceded the attempt's **first measured sample**, which is a fact about the run rather than about
/// the lifecycle stage: [`Self::Sample`] straddles it, since a channel can fail before its first
/// sample confirms or after several have. Treating this enum as the boundary would make the rule
/// wrong in exactly the case it exists to govern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum InfrastructurePhase {
    /// Resolving the distribution, staging the module, starting the server, or publishing. The
    /// attempt never reached a measurable state.
    Provision,
    /// Connecting the measured subscriber.
    Connect,
    /// Applying the attempt's deterministic seeding writes, before any measured channel opens. Its
    /// own phase because a seeding write is neither provisioning — the server is already published
    /// and connected — nor a measured sample.
    Seeding,
    /// Subscribing, or awaiting the applied snapshot.
    Subscription,
    /// Issuing a measured write, or sealing a channel's batch.
    Sample,
    /// Durably retaining an observed row set for composition validation.
    ObservationRetention,
}
