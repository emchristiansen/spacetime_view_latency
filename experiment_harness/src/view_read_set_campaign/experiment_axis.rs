//! Which frozen finite range a scale point varies.

use serde::Serialize;

/// Which of the spec's frozen finite ranges an attempt's scale point is drawn from.
///
/// The spec names four applicable axes across its fourteen candidate/axis pairs; only the one this
/// stage executes is declared. A variant added later must not reinterpret evidence already written
/// under this vocabulary, and arrives together with the driver that can measure it.
///
/// The axis is retained inside every [`ScalePoint`](super::scale_point::ScalePoint) even though it
/// admits one value: evidence that does not name its axis cannot be separated correctly once a
/// second axis exists, and a rung index alone would not say which ladder it indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ExperimentAxis {
    /// Rows held by an identity other than the measured subscriber — they grow the backing table
    /// without growing the measured result set.
    UnrelatedGlobalRows,
}

impl ExperimentAxis {
    /// A stable canonical token naming this axis, for the identity-derived artifact directory.
    ///
    /// Frozen independently of the variant name, exactly as
    /// [`CandidateId::canonical_tag`](super::candidate_id::CandidateId::canonical_tag) is, so a
    /// later rename cannot move the directory a recorded finding points into.
    pub(crate) fn canonical_tag(self) -> &'static str {
        match self {
            Self::UnrelatedGlobalRows => "unrelated-global-rows",
        }
    }
}
