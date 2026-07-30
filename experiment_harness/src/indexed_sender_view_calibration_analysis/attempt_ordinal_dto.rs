//! The untrusted wire form of an attempt's ordinal at its logical slot.

use serde::Deserialize;

/// The wire form of
/// [`AttemptOrdinal`](crate::indexed_sender_view_calibration_pilot::attempt_ordinal::AttemptOrdinal).
///
/// Mirrored and matched. §568 forbids a hidden retry, and the pilot makes one unrepresentable by
/// giving this type a single variant — but an unvalidated ordinal on the read side would let a
/// ledger claiming a retried attempt be admitted as though it were an original, which is exactly the
/// fact the freeze exists to make visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum AttemptOrdinalDto {
    /// The first and only attempt at this logical slot.
    Original,
}
