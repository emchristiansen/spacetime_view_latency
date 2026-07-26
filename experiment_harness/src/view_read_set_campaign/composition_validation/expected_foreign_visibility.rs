//! How much of the foreign slice a role is required to see.

use serde::Serialize;

/// How many of the other identity's rows an attempt's role must observe.
///
/// A closed sum rather than a count, because the two cases are not two numbers — they are two
/// different claims. [`Self::None`] is the Arm's security gate: the sender-scoped view returning
/// even one foreign row is a leak, not a small discrepancy. [`Self::All`] is the Control's
/// composition claim: the base-table subscription returning fewer than the whole seeded slice means
/// the backing table was not composed as preregistered.
///
/// Keeping them distinct stops the security gate from being written as a numeric comparison a later
/// refactor could relax by one, and makes it a case a reader must handle rather than a threshold
/// they could tune.
///
/// Freely constructible, and deliberately so: this is part of an *expectation*, which asserts
/// nothing about what was observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ExpectedForeignVisibility {
    /// None of the foreign slice may appear. The Arm's sender-scoped view under its security gate.
    None,
    /// The whole seeded foreign slice must appear, and this is its exact size. The Control's
    /// direct-table subscription, which is what makes it the composition baseline rather than an
    /// independent measurement.
    All { rows: u64 },
}
