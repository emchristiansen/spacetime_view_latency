//! A paced series that was recorded but never admitted, retained in full.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::paced_sample_nanos::PacedSampleNanos;

/// The ordered raw latencies of an attempt whose series was never sealed.
///
/// **Retention is mandatory, and partiality is real here.** The spec requires raw per-sample
/// retention, so an attempt keeps every nanosecond it produced even when the host bracket could not
/// be closed, the paced batch stopped partway, the composition mismatched, or a sample was
/// nonpositive — the last case retaining a literal zero, which is exactly the observation a
/// calibration ledger must show.
///
/// This differs from the discovery screen's rejected sample in one structural way. E3 measures a
/// single cold apply, so a failed measurement has no sample *by construction*. A paced batch is up
/// to a thousand samples issued one at a time, so a batch that fails at sample 400 genuinely holds
/// four hundred — and those four hundred are informative about exactly the stationarity and lag
/// questions this pilot exists to answer. Discarding them because the attempt failed would throw
/// away most of what it learned.
///
/// **The visibility boundary is the same one, with the same honest limit.** The samples are
/// [`PacedSampleNanos`], whose field is private to its own module, so this type can hold and
/// serialize them while being unable to read them directly — and there is no ordinary path from a
/// rejected series to
/// [`CalibrationSeries::recorded`](super::calibration_series::CalibrationSeries::recorded), which
/// takes freshly minted samples plus a verifier-minted population token. `Serialize` is still
/// required, so this is not a capability boundary; see [`PacedSampleNanos`] for the exact statement.
///
/// **No `Debug`**, propagated from [`PacedSampleNanos`], and carried up through every containing
/// type.
#[derive(Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct RejectedSeries(Vec<PacedSampleNanos>);

impl RejectedSeries {
    /// Retain what an attempt produced, as non-evidence.
    ///
    /// Accepts an empty vector without complaint, unlike
    /// [`CalibrationSeries::recorded`](super::calibration_series::CalibrationSeries::recorded): a
    /// batch that failed before its first sample completed has an empty series, and
    /// [`PartialEvidence`](super::partial_evidence::PartialEvidence) distinguishes "no series at
    /// all" from "a series that happens to be empty" by which variant it uses, not by this
    /// constructor refusing one.
    pub(crate) fn of(samples: Vec<PacedSampleNanos>) -> Self {
        Self(samples)
    }

    /// How many samples were retained.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}
