//! What a failed attempt observed before it failed.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::composition_mismatch::CompositionMismatch;
use crate::indexed_sender_view_calibration_pilot::rejected_series::RejectedSeries;

/// The evidence a failed attempt genuinely holds, including the ordered raw samples of a paced batch
/// that ran but was never admitted.
///
/// **Rejected series are retained.** The spec requires raw per-sample retention, so an attempt keeps
/// every nanosecond it produced even when the batch stopped early, the host bracket could not be
/// closed, the composition mismatched, or a sample was nonpositive — the last case retaining a
/// literal zero.
///
/// **Retention rests on a visibility boundary rather than a convention.** The samples are held as
/// [`RejectedSeries`] over [`PacedSampleNanos`](super::paced_sample_nanos::PacedSampleNanos), whose
/// field is private to its own module. These variants are `pub(crate)` and so are their fields —
/// Rust gives variant fields the enum's visibility and offers no way to narrow them — but
/// destructuring one yields an opaque series this module cannot read either, so no ordinary path
/// leads from a rejected series to
/// [`CalibrationSeries::recorded`](super::calibration_series::CalibrationSeries::recorded), which
/// takes freshly minted samples plus a verifier-minted population token. That is a boundary against
/// the accidental path, not against a determined serde round trip; see [`PacedSampleNanos`].
///
/// The mismatch *is* readable, because the diagnostic content of a composition failure is precisely
/// which rows diverged and how, and a mismatch carries no duration to build a statistic from.
///
/// **No `Debug`**, propagated from the retained samples, and carried up through every containing
/// type.
#[derive(Clone, Serialize)]
pub(crate) enum PartialEvidence {
    /// The attempt failed before its first append completed. There is no series.
    NothingObserved,
    /// The batch produced samples; the caches were never read. Reached when the batch stopped early
    /// or the post-measurement host observation failed.
    ///
    /// The series may be empty — a batch that failed on its very first append began measuring and
    /// recorded nothing. That is deliberately distinct from [`Self::NothingObserved`], which means
    /// the attempt never reached the batch at all: "measured and got nothing" and "never measured"
    /// are different facts, and collapsing them would lose which one a ledger line describes.
    RejectedSeries { series: RejectedSeries },
    /// The batch produced samples and both caches were read, and the composition did not hold.
    /// Retained in full, because which rows diverged is the diagnostic content of a semantic
    /// failure.
    RejectedSeriesAndMismatch {
        series: RejectedSeries,
        mismatch: CompositionMismatch,
    },
}

impl PartialEvidence {
    /// Whether a paced batch's raw samples are retained here.
    ///
    /// Answers only the yes/no question the cross-field invariants need, and deliberately does not
    /// hand back the numbers. True even for a retained empty series: the question is whether the
    /// batch ran, not whether it produced anything.
    pub(crate) fn has_series(&self) -> bool {
        match self {
            Self::NothingObserved => false,
            Self::RejectedSeries { .. } | Self::RejectedSeriesAndMismatch { .. } => true,
        }
    }

    /// How many samples were retained, when a batch ran at all.
    pub(crate) fn retained_samples(&self) -> Option<usize> {
        match self {
            Self::NothingObserved => None,
            Self::RejectedSeries { series } => Some(series.len()),
            Self::RejectedSeriesAndMismatch { series, .. } => Some(series.len()),
        }
    }

    /// The composition mismatch this attempt found, when it got as far as reading the caches.
    pub(crate) fn mismatch(&self) -> Option<&CompositionMismatch> {
        match self {
            Self::NothingObserved | Self::RejectedSeries { .. } => None,
            Self::RejectedSeriesAndMismatch { mismatch, .. } => Some(mismatch),
        }
    }
}
