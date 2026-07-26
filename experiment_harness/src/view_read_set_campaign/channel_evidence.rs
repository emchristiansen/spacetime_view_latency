//! One measured channel's lossless evidence and the statistic it reduces to.

use anyhow::Result;
use serde::Serialize;

use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::view_read_set_campaign::cell_statistic::CellStatistic;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::saturated_timing_batch::SaturatedTimingBatch;

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
/// **Phase 1 boundary.** The variants, their payload shapes, and the shape validation on the way in
/// are complete; the four reductions are explicit `todo!()` stubs until Phase 2.
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
    pub(crate) fn saturated(timings: SaturatedTimingBatch) -> Result<Self> {
        let _ = timings;
        todo!(
            "Phase 2: take the exact all-pairs Theil-Sen slope over (issue index, latency) using \
             crate::analysis::stats::theil_sen::theil_sen_slope, then admit it through \
             CellStatistic::validated so a nonpositive slope fails the attempt"
        )
    }

    /// Reduce a paced batch to the exact rational median of its samples.
    pub(crate) fn paced(samples: RawLatencies) -> Result<Self> {
        let _ = samples;
        todo!(
            "Phase 2: take the exact rational median using crate::analysis::stats::median::median \
             (the batch size is even, so this is the exact mean of the two central order \
             statistics), then admit it through CellStatistic::validated"
        )
    }

    /// Take the cold-subscription apply duration as its own statistic.
    pub(crate) fn cold_subscription(applied: LatencySample) -> Result<Self> {
        let _ = applied;
        todo!(
            "Phase 2: admit the apply duration through CellStatistic::validated, so a \
             zero-nanosecond apply is refused rather than treated as instantaneous"
        )
    }

    /// Take the reconnect apply duration as its own statistic.
    pub(crate) fn reconnect(applied: LatencySample) -> Result<Self> {
        let _ = applied;
        todo!("Phase 2: as ChannelEvidence::cold_subscription, for the reconnect window")
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
