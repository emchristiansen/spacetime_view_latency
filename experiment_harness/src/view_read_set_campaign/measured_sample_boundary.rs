//! Whether a failed attempt had already taken a measured sample.

use serde::Serialize;

/// Which side of its **first measured sample** an attempt failed on.
///
/// This is the timing term of the spec's retry rule — "infrastructure failure occurring before its
/// first measured sample" — and it cannot be recovered from the retained evidence:
///
/// - An empty channel prefix is ambiguous. A failure while issuing the very first write of the first
///   channel leaves exactly the same empty prefix as a failure before any measurement was attempted,
///   and those fall on opposite sides of the rule.
/// - A retained composition finding does not settle it either. The spec freezes the order of the
///   four channels but not where composition validation sits among them, so a nonempty composition
///   with an empty channel prefix says nothing about whether a sample was taken.
///
/// Inferring it from evidence would therefore be guessing, and a retry criterion that guesses is one
/// that can reference a measured outcome by accident — precisely what the contract forbids.
///
/// **Where the value comes from.** [`FailureStage::measured_sample_boundary`](super::failure_stage::FailureStage::measured_sample_boundary),
/// a total match over the driver-stated stage that the terminal record actually carries. It is
/// derived rather than stored alongside that stage because the two would otherwise be able to
/// contradict each other — "never published, but measured" is a well-typed pair of fields and an
/// unreachable state. The stage is what is serialized; a reader recomputes this from it, exactly as
/// the retry decision is recomputed rather than trusted.
///
/// A closed pair rather than a `bool` because the field is read by a rule where the two cases have
/// opposite consequences; a bare boolean at that call site would be a coin whose sides are unlabelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum MeasuredSampleBoundary {
    /// The attempt failed before any measured sample was taken. The retry-eligible side, for an
    /// infrastructure cause.
    BeforeFirst,
    /// At least one measured sample had been taken. Never retry-eligible, whatever the cause: past
    /// this point a retry criterion would be referencing a measured outcome.
    AfterFirst,
}
