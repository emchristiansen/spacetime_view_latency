//! What kind of failure terminated an attempt.

use serde::Serialize;

/// The closed set of failure causes that terminate a Pilot attempt.
///
/// The Parked Frontier fixes the terminating causes: "any subscription/reducer/sample error
/// terminates that attempt while preserving partial evidence; later inventory attempts continue".
/// These variants are those causes plus the two lifecycle boundaries that can fail before or after
/// the measured window — provisioning the isolated server, and tearing it down.
///
/// Classifying the cause is what lets a later reader distinguish an infrastructure failure from a
/// candidate failure without re-reading raw diagnostics. The free-text detail lives in
/// [`DiagnosticArtifact`](super::diagnostic_artifact::DiagnosticArtifact); this enum is the part
/// that must stay machine-readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailureKind {
    /// Resolving the pinned distribution, staging the module, starting the isolated server, or
    /// publishing the module failed. The attempt never reached a measurable state.
    Provision,
    /// Connecting the measured subscriber failed.
    Connect,
    /// Seeding a row through its insertion reducer failed, or a reducer returned an error.
    Reducer,
    /// Subscribing to the attempt's target, or awaiting its applied snapshot, failed.
    Subscription,
    /// A measured write failed to confirm, a confirmation callback reported an error, or the
    /// measured batch could not be sealed into a full sample vector.
    Sample,
    /// A semantic or authorization check over the observed result set failed — for the Arm, the
    /// sender-scoped view returning anything other than exactly the measured identity's own rows.
    Semantics,
    /// Disconnecting the client or tearing the isolated server down failed after the measured work.
    /// Evidence already collected remains valid; the attempt is still recorded as failed because the
    /// isolation guarantee for what follows is no longer established.
    Teardown,
}
