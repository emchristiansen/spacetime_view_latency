//! Whether the first measured sample had begun when an attempt failed.

use serde::Serialize;

/// How far an attempt had got when it failed, relative to its first measured sample.
///
/// The second half of the spec's retry rule, which is prospective: an infrastructure failure is
/// retryable only when it occurs *before* the first measured sample. Once sampling has begun, a
/// retry would replace a measurement already in progress rather than replacing an attempt that never
/// measured anything — so this is recorded as a fact about the attempt rather than inferred later
/// from the failure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum SamplingProgress {
    /// Nothing was measured; the attempt failed on the way to its first sample.
    BeforeFirstSample,
    /// The timed interval had begun, so a measurement was already underway.
    AtOrAfterFirstSample,
}
