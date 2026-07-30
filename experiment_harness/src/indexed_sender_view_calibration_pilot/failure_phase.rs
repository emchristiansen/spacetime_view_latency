//! Which side of the spec's retry rule a failure falls on.

use serde::Serialize;

/// Which settlement bucket a failure falls in, for the sole purpose of deciding retry eligibility.
///
/// **This is a settlement-policy classification, not a claim about physical causation.** The spec
/// draws retry eligibility along exactly one line: a *prospective* infrastructure failure at
/// provisioning or connection is an interruption of measurement that may be silently replaced, while
/// everything else is an outcome that must be recorded rather than re-rolled until it came out
/// favourably. This enum names those buckets and nothing more; it does not partition failures by
/// what actually went wrong.
///
/// **The typed physical cause is therefore not recoverable from this field.**
/// [`FailureKind`](super::failure_kind::FailureKind) narrows it only to a lifecycle location and
/// category — `PacedBatch`, for instance, names where the attempt stopped, not whether a reducer
/// refused, a barrier timed out, or a channel dropped. Below that, the retained diagnostic *text* is
/// the only surviving causal detail. That gap is real and shared with the accepted E3 screen — see
/// [`AttemptFailure`](super::attempt_failure::AttemptFailure) — and is harmless to this pilot's only
/// output, because every failure blocks `W` selection identically regardless of cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailurePhase {
    /// The prospective-retry class, and only that: provisioning or connection failed before any
    /// measurement began. The only phase a retry can be earned from, and only before the first
    /// measured sample.
    Infrastructure,
    /// The residual non-retryable bucket: everything the retry policy refuses to re-roll.
    ///
    /// **Not an assertion that the system under test produced every member**, despite the name. It
    /// deliberately *includes* infrastructure-shaped and host-observation-shaped faults, because
    /// policy — not causation — decides membership. `PacedBatch` covers barrier timeouts and dropped
    /// channels as readily as reducer refusals, and `HostObservationAfterBatchFailure` is a `/proc`
    /// read fault that lands here rather than in [`Self::HarnessObservation`], because its causally
    /// primary failure is the batch stopping early. Only `Provision` and `Connect` occupy
    /// [`Self::Infrastructure`]; a failure that is not one of those and is not a standalone
    /// host-observation fault lands here whatever caused it. Never retryable, whenever it occurred.
    ApplicationOrSemantic,
    /// The harness could not read the host observations the frozen method requires around a
    /// measurement, and that read *was* the primary failure.
    ///
    /// Exactly `HostObservationBefore` and `HostObservationAfter`, and no other kind. Its own bucket
    /// because it is neither of the others: `/proc` becoming unreadable is not the system under test
    /// producing a result, but neither is it a prospective interruption a retry may silently replace.
    /// The spec makes both non-retryable, and a distinct bucket records that as a fact about the
    /// failure rather than filing a standalone harness fault under a label naming the application.
    ///
    /// It does **not** collect every host-observation fault: `HostObservationAfterBatchFailure` is
    /// one, but its primary failure is the batch stopping early, so it settles as
    /// [`Self::ApplicationOrSemantic`]. Classifying it by the later `/proc` fault would let one
    /// measurement outcome change bucket according to whether a subsequent read happened to succeed.
    HarnessObservation,
}
