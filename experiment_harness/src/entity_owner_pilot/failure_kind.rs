//! What kind of failure terminated an attempt.

use serde::Serialize;

/// The classified cause that terminated an attempt.
///
/// The spec's terminating causes (subscription, reducer, sample) plus the lifecycle boundaries that
/// can fail outside the measured window. Classifying the cause lets a reader distinguish an
/// infrastructure failure from a candidate failure without parsing raw diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailureKind {
    /// Resolving the distribution, staging the module, starting the server, or publishing failed.
    /// The attempt never reached a measurable state.
    Provision,
    /// Connecting the measured subscriber failed.
    Connect,
    /// A seeding insert failed, or a reducer returned an error.
    Reducer,
    /// Subscribing, or awaiting the applied snapshot, failed.
    Subscription,
    /// A measured write failed to confirm, or the batch could not be sealed into a full sample
    /// vector.
    Sample,
    /// A semantic or authorization check over the observed result set failed — for the Arm, the
    /// sender-scoped view returning anything but exactly the measured identity's own rows.
    Semantics,
    /// Disconnecting or tearing the server down failed after the measured work. Evidence already
    /// collected stays valid; the attempt is still failed because isolation for what follows is no
    /// longer established.
    Teardown,
}
