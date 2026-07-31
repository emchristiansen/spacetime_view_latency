//! Why a predeclared attempt was never executed.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;

/// The reasons a frozen slot can terminate without being attempted at all.
///
/// `NotRun` means the slot never entered acquisition — never that it tried and failed. A failure
/// after the gate admits is a failure record in one of the four stage-specific shapes, carrying
/// whatever facts existed.
///
/// **A refusal here costs a replicate, and that is worth stating explicitly.** The spec's decision
/// rule needs *both* complete series, so an attempt settled `EnvironmentRefused` consumes no
/// measurement or retry budget yet leaves the pilot unable to deliver what it was authorized to
/// deliver. The spec is direct about the consequence: if either attempt is absent, Control redesigns
/// or defers rather than manufacturing `W` from the survivor.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum NotRunReason {
    /// The host gate ran and refused this attempt. It consumes no measurement or retry budget and
    /// does not abort the remaining slot — this slot settles here and the pilot moves on. It does,
    /// however, consume the slot itself: this replicate yields no series in this run.
    EnvironmentRefused,
    /// The host gate reached no verdict, so nothing was gated.
    ///
    /// Two ways in: a waiter that could not be spawned at all, and one that ran but terminated on an
    /// unexpected code or a signal instead of admitting or refusing. Neither established a gate
    /// verdict, so the run cannot claim the attempt was admitted or refused — which is what
    /// separates this from `EnvironmentRefused`.
    ///
    /// Terminal for the run rather than for this slot alone — not as a prediction that the fault is
    /// permanent, but because this run can no longer show that what it would measure next was gated.
    GateInoperable { diagnostic: DiagnosticArtifact },
    /// A preceding attempt did not provably release every resource it acquired, so this slot was
    /// never executed.
    ///
    /// Named for what was observed — a release step that failed — rather than for a specific
    /// consequence such as a surviving server, because the aggregating teardown cannot separate a
    /// staged-file cleanup failure from a server-shutdown one.
    PriorAttemptReleaseFailed { diagnostic: DiagnosticArtifact },
}
