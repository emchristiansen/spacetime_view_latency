//! One measured channel's lossless evidence and the statistic it reduces to.

use anyhow::{Context, Result};
use serde::Serialize;

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::theil_sen::theil_sen_slope;
use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::view_read_set_campaign::cell_statistic::CellStatistic;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::saturated_timing_batch::SaturatedTimingBatch;

/// One apply duration as the exact rational number of nanoseconds it lasted.
///
/// Shared by the two single-sample channels so their one-line reductions cannot drift apart in how
/// they convert a sample — the difference between E3 and E4 is which window was measured, never how
/// the number was formed.
fn apply_duration(applied: LatencySample) -> Rational {
    Rational::new(
        i128::try_from(applied.nanos()).expect("a measured latency in nanoseconds fits i128"),
        1,
    )
}

/// One channel's complete evidence at one scale point: what it observed, and the `S` it reduced to.
///
/// One variant per channel, each carrying only the sample shape that channel can actually produce,
/// so a saturated batch paired with a single apply duration — or an apply channel paired with a
/// batch — is not a state that exists. The channel is *read off* the variant ([`Self::channel`])
/// rather than stored beside it, so there is no second copy of that fact to disagree with the
/// payload.
///
/// Each constructor performs its own channel's reduction rather than accepting a pre-computed
/// statistic, so the wrong reduction cannot be applied to the right samples by caller discipline.
/// Every one yields a [`CellStatistic`], which admits only strictly positive exact rationals — so a
/// channel that reduced to zero or a negative value fails its attempt at that door instead of
/// reaching classification and being reported as flat.
///
/// The lossless observations are retained *alongside* the statistic rather than replaced by it, so
/// every reported `S` stays recomputable from the record and a later reduction convention can be
/// applied to the same evidence.
///
/// What this does *not* guarantee: that the observations are genuine. The constructors are
/// `pub(crate)` and will accept any well-formed batch, so the type binds channel, sample shape, and
/// reduction to each other — nothing more. Genuineness is a property of the driver's measurement
/// path, as it is for
/// [`RawLatencies`](crate::observation::raw_latencies::RawLatencies).
///
/// The variants, their payload shapes, the shape validation on the way in, and the four reductions
/// are all complete. What produces the observations — the measured window against a live server —
/// is the driver's measurement stage, which is still a `todo!()`.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum ChannelEvidence {
    /// Marginal per-write cost under a saturated pipeline. Retains both offsets per write rather
    /// than only their difference, so the FIFO and stable-issue-spacing interpretation the slope
    /// depends on stays falsifiable against the record.
    SaturatedQueueGrowthPerWrite {
        timings: SaturatedTimingBatch,
        slope: CellStatistic,
    },
    /// Paced visible-apply latency: one batch of samples, each taken with a single outstanding
    /// write, reduced to their exact rational median.
    PacedVisibleApplyLatency {
        samples: RawLatencies,
        median: CellStatistic,
    },
    /// The one cold-subscription apply duration this fresh server can yield.
    ColdSubscriptionApplyTime {
        applied: LatencySample,
        duration: CellStatistic,
    },
    /// The one token-preserving reconnect-and-resubscribe apply duration.
    ReconnectApplyTime {
        applied: LatencySample,
        duration: CellStatistic,
    },
}

impl ChannelEvidence {
    /// Reduce a sealed saturated batch to its exact all-pairs Theil–Sen slope of latency against
    /// issue index — the marginal per-write cost.
    /// The abscissa is the *issue index*, not the issue offset. The spec's estimand is the marginal
    /// cost per additional queued write, so the slope is taken against position in the batch; the
    /// recorded offsets stay in the batch precisely so that reading can be falsified rather than
    /// assumed.
    pub(crate) fn saturated(timings: SaturatedTimingBatch) -> Result<Self> {
        let points: Vec<(i128, i128)> = timings
            .writes()
            .iter()
            .enumerate()
            .map(|(index, timing)| {
                let issue_index = i128::try_from(index).expect("a saturated batch index fits i128");
                let latency = i128::try_from(timing.latency_nanos())
                    .expect("a measured latency in nanoseconds fits i128");
                (issue_index, latency)
            })
            .collect();
        let slope = CellStatistic::validated(theil_sen_slope(&points))
            .context("reducing a saturated batch to its Theil–Sen slope")?;
        Ok(Self::SaturatedQueueGrowthPerWrite { timings, slope })
    }

    /// Reduce a paced batch to the exact rational median of its samples.
    ///
    /// The batch size is even, so this is the exact mean of the two central order statistics — a
    /// rational that is generally not an integer number of nanoseconds, which is why the statistic
    /// is carried as a [`Rational`] rather than rounded here.
    pub(crate) fn paced(samples: RawLatencies) -> Result<Self> {
        let values: Vec<Rational> = samples
            .samples()
            .iter()
            .map(|sample| {
                Rational::new(
                    i128::try_from(sample.nanos())
                        .expect("a measured latency in nanoseconds fits i128"),
                    1,
                )
            })
            .collect();
        let median = CellStatistic::validated(median(&values))
            .context("reducing a paced batch to its exact rational median")?;
        Ok(Self::PacedVisibleApplyLatency { samples, median })
    }

    /// Take the cold-subscription apply duration as its own statistic.
    ///
    /// Measured exactly once per attempt, so there is nothing to reduce *across* — the admission
    /// through [`CellStatistic::validated`] is the whole reduction, and it is what refuses a
    /// zero-nanosecond apply rather than reporting it as instantaneous.
    pub(crate) fn cold_subscription(applied: LatencySample) -> Result<Self> {
        let duration = CellStatistic::validated(apply_duration(applied))
            .context("admitting the cold-subscription apply duration")?;
        Ok(Self::ColdSubscriptionApplyTime { applied, duration })
    }

    /// Take the reconnect apply duration as its own statistic.
    pub(crate) fn reconnect(applied: LatencySample) -> Result<Self> {
        let duration = CellStatistic::validated(apply_duration(applied))
            .context("admitting the reconnect apply duration")?;
        Ok(Self::ReconnectApplyTime { applied, duration })
    }

    /// Which channel this evidence belongs to, read off the variant.
    pub(crate) fn channel(&self) -> MeasurementChannel {
        match self {
            Self::SaturatedQueueGrowthPerWrite { .. } => {
                MeasurementChannel::SaturatedQueueGrowthPerWrite
            }
            Self::PacedVisibleApplyLatency { .. } => MeasurementChannel::PacedVisibleApplyLatency,
            Self::ColdSubscriptionApplyTime { .. } => MeasurementChannel::ColdSubscriptionApplyTime,
            Self::ReconnectApplyTime { .. } => MeasurementChannel::ReconnectApplyTime,
        }
    }

    /// The reduced, strictly positive `S` this channel contributes to its endpoint factor.
    pub(crate) fn statistic(&self) -> CellStatistic {
        match self {
            Self::SaturatedQueueGrowthPerWrite { slope, .. } => *slope,
            Self::PacedVisibleApplyLatency { median, .. } => *median,
            Self::ColdSubscriptionApplyTime { duration, .. } => *duration,
            Self::ReconnectApplyTime { duration, .. } => *duration,
        }
    }
}

#[cfg(test)]
mod tests;
