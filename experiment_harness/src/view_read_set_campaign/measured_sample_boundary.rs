//! Whether a failed attempt had already taken a measured sample.

use serde::Serialize;

/// Which side of its **first measured sample** an attempt failed on.
///
/// This is the timing term of the spec's retry rule — "infrastructure failure occurring before its
/// first measured sample" — and it is recorded rather than derived because it genuinely cannot be
/// recovered from the retained evidence:
///
/// - An empty channel prefix is ambiguous. A failure while issuing the very first write of the first
///   channel leaves exactly the same empty prefix as a failure before any measurement was attempted,
///   and those fall on opposite sides of the rule.
/// - A retained composition finding does not settle it either. The spec freezes the order of the
///   four channels but not where composition validation sits among them, so a nonempty composition
///   with an empty channel prefix says nothing about whether a sample was taken.
///
/// Inferring it would therefore be guessing, and a retry criterion that guesses is one that can
/// reference a measured outcome by accident — precisely what the contract forbids. So the driver,
/// which is the only thing that knows, states it, and the terminal record carries it so a reader can
/// audit the retry decision rather than re-derive it.
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
