//! Why a predeclared attempt was never executed.

use serde::Serialize;

use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;

/// Why a predeclared attempt produced no execution.
///
/// Without this, a stopped campaign would simply be missing lines, and a missing line is
/// indistinguishable from one that was never planned — which is what makes "report generation fails
/// on missing or duplicate planned identities" checkable.
///
/// The one variant is the one this stage can both produce *and record*. A ledger that fails to
/// persist cannot append a truthful record about its own failure, so that case is detected by
/// diffing the durable frozen inventory against the terminal records present, rather than by a
/// variant no code path could ever write.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum NotRunReason {
    /// A preceding attempt did not provably release every resource it acquired, so this slot was
    /// never executed.
    ///
    /// Named for what was actually observed — a release step that failed — rather than for a
    /// specific consequence such as a surviving server. Release runs as one aggregating step over
    /// the client, the server, and the staged artifact, so a staged-file cleanup failure is not
    /// separable from a server-shutdown one; under fail-fast semantics any of them is
    /// campaign-terminal, because continuing on the assumption that the *unproven* part was the
    /// harmless one is exactly the assumption this campaign must not make.
    ///
    /// This matters more here than it did for a cumulative walk: every scale point is a separately
    /// provisioned fresh server binding the same listen address, so an unreleased server would be
    /// inherited by the very next attempt.
    ///
    /// The failing release's diagnostic is carried here rather than referenced by attempt identity,
    /// so a reader learns why a slot was skipped without having to locate and interpret the
    /// preceding attempt's own terminal record.
    PriorAttemptReleaseFailed { diagnostic: DiagnosticArtifact },
}
