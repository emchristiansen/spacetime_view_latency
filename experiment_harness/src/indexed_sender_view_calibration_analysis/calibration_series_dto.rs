//! The untrusted wire form of a sealed calibration series.

use serde::Deserialize;

use crate::indexed_sender_view_calibration_analysis::verified_population_dto::VerifiedPopulationDto;

/// The wire form of
/// [`CalibrationSeries`](crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries).
///
/// **This is the deliberate serde route the pilot documents as remaining open**, now taken for the
/// stated §569 purpose that module says such parsing code would otherwise lack. No accessor was added
/// to `CalibrationSeries` or to `PacedSampleNanos`; the numbers are recovered here, at an external
/// boundary, exactly as the pilot's own rustdoc says a determined reader could.
///
/// **The sample shape is proven, not assumed.** `CalibrationSeries` is a `#[derive(Serialize)]`
/// struct with fields `samples: Vec<PacedSampleNanos>` and `population: VerifiedPopulation`, and
/// `PacedSampleNanos` is `#[serde(transparent)]` over a `u128`. So `samples` is a bare JSON array of
/// integers, not an array of objects.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CalibrationSeriesDto {
    /// Every retained raw nanosecond, in issue order. Order is the evidence — §569's window,
    /// trend, and lag diagnostics are all statements about position — so this is never sorted here.
    pub(crate) samples: Vec<u128>,
    /// The composition proof the series was sealed against.
    pub(crate) population: VerifiedPopulationDto,
}
