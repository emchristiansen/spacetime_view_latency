//! One row as the composition verifier sees it.

use spacetimedb_sdk::Identity;

/// A single `IndexedControlActivity` row, flattened to the four columns the table actually has.
///
/// Deliberately a harness-side snapshot rather than the generated binding row. Two reasons, and the
/// second is the one that matters:
///
/// - the generated type is exact-version-coupled plumbing, and the verifier's rules are a property
///   of the frozen composition rather than of this month's bindings;
/// - a plain value with public fields can be constructed in a focused test, so every rule below —
///   wrong owner at the right count, a duplicated id, a timestamp off by one step — is provable
///   without a live server. A verifier that could only be exercised against a running instance is a
///   verifier nothing checks.
///
/// `ts` is carried as raw microseconds since the Unix epoch rather than a `Timestamp`, because the
/// frozen recipe *derives* it arithmetically from the id and the check is exact equality against
/// that derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObservedRow {
    pub(crate) id: u64,
    pub(crate) ts_micros: i64,
    pub(crate) control_uuid: u64,
    pub(crate) user_identity: Identity,
}
