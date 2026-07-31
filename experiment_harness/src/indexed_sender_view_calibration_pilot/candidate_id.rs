//! The candidate this pilot calibrates a method for.

use serde::Serialize;

/// The single candidate this pilot touches.
///
/// **One variant, and that is a constraint rather than a stub.** The spec's calibration ceiling
/// forbids an Arm/Control comparison, and a comparison needs two things to compare. With one
/// candidate and one role in the whole vocabulary, the forbidden contrast has no values it could be
/// formed from — it is unrepresentable rather than merely unwritten.
///
/// The wider `CandidateId` vocabulary of the governing spec is deliberately not restated: this enum
/// carries only what this module writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum CandidateId {
    /// The indexed `control_activity` sender view — the candidate whose append-only E2 estimand the
    /// screen this pilot calibrates will measure.
    IndexedControlActivitySenderView,
}
