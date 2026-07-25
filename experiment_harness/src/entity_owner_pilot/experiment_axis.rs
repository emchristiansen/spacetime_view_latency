//! Which frozen finite range an attempt varies.

use serde::Serialize;

/// Which of the spec's frozen finite ranges an attempt walks.
///
/// The spec names the `axis: ExperimentAxis` field of
/// [`AttemptKey`](super::attempt_key::AttemptKey) but never enumerates its variants; rather than
/// invent that enumeration, only the axis this Pilot walks is declared. A variant added later must
/// not reinterpret evidence already written under this vocabulary.
///
/// The field is retained on `AttemptKey` even though it admits one value: evidence that does not
/// name its axis cannot be separated correctly once a second axis exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ExperimentAxis {
    /// Rows held by an identity other than the measured subscriber — they grow the backing table
    /// without growing the measured result set.
    UnrelatedGlobalRows,
}
