//! Why a predeclared attempt was never executed.

use serde::Serialize;

use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;

/// The reasons a frozen slot can terminate without being attempted at all.
///
/// `NotRun` means the slot never entered acquisition — never that it tried and failed. A failure
/// after the gate admits is a `Failed` record in one of the four stage-specific shapes, carrying
/// whatever facts existed; conflating the two would let a provisioning failure masquerade as an
/// unattempted slot and disappear from the retryable-failure accounting.
///
/// A truthful `NotRun` inventory is itself evidence supporting an indeterminate status, which is why
/// an unreached slot is recorded rather than skipped, and why the two terminal reasons carry the
/// diagnostic that stopped the run rather than referring the reader to another record.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum NotRunReason {
    /// The host gate ran and refused this attempt. It consumes no measurement or retry budget and
    /// does not abort the remaining slots — this slot settles here and the screen moves on.
    EnvironmentRefused,
    /// The host gate could not be executed at all, so nothing was gated.
    ///
    /// Terminal for the run rather than for this slot alone: a waiter that cannot be spawned will
    /// not gate any later attempt either, and measuring on an ungated host is exactly what the gate
    /// exists to prevent. The current slot and every remaining one are recorded with this reason and
    /// flushed before the screen stops.
    GateInoperable { diagnostic: DiagnosticArtifact },
    /// A preceding attempt did not provably release every resource it acquired, so this slot was
    /// never executed.
    ///
    /// Named for what was observed — a release step that failed — rather than for a specific
    /// consequence such as a surviving server, because the aggregating teardown cannot separate a
    /// staged-file cleanup failure from a server-shutdown one, and continuing on the assumption that
    /// the *unproven* part was the harmless one is precisely the assumption this screen must not
    /// make. Copies the Pilot's `PriorAttemptReleaseFailed` for the same reason it exists there.
    ///
    /// The failing release is *also* retained on the attempt that produced it, via that record's
    /// own `ResourceDisposition`, so a release failure on the final slot is not lost for want of a
    /// successor to carry it.
    PriorAttemptReleaseFailed { diagnostic: DiagnosticArtifact },
}
