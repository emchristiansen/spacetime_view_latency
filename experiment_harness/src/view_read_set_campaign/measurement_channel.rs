//! Which of the four measured channels a cell statistic belongs to.

use serde::Serialize;

/// The four measurement channels required for every applicable candidate/axis pair.
///
/// Each channel is a separate estimand answering a different question, so they are a closed
/// enumeration rather than a parameter: a saturated pipeline cannot answer a latency question, and a
/// cold subscription cannot be re-measured on a server that is no longer cold.
///
/// A channel classifies independently, and any one of them classifying proportional-or-worse blocks
/// its candidate — so the report cannot select whichever channel happened to look favorable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum MeasurementChannel {
    /// Marginal per-write cost under a saturated write pipeline: the exact all-pairs Theil–Sen
    /// slope of latency against issue index over exactly one batch of issued writes.
    SaturatedQueueGrowthPerWrite,
    /// One outstanding write at a time, stopped when the change is observable in the subscriber
    /// cache: the exact rational median of one batch of paced samples.
    PacedVisibleApplyLatency,
    /// A fresh connection subscribing at this scale point, stopped when the initial snapshot is in
    /// cache: one apply duration.
    ColdSubscriptionApplyTime,
    /// Token-preserving reconnect and resubscribe, stopped at applied: one apply duration.
    ReconnectApplyTime,
}

impl MeasurementChannel {
    /// The frozen execution order every attempt runs its channels in.
    ///
    /// The order is preregistered rather than incidental, and each position is forced by the one
    /// before it: the cold subscription must be first because nothing else may have subscribed yet;
    /// the reconnect follows because it needs that connection to reconnect *from*; the paced channel
    /// reuses it; and the saturating channel runs last, where its queue pressure cannot perturb a
    /// channel that has not yet been measured.
    pub(crate) const EXECUTION_ORDER: [MeasurementChannel; 4] = [
        MeasurementChannel::ColdSubscriptionApplyTime,
        MeasurementChannel::ReconnectApplyTime,
        MeasurementChannel::PacedVisibleApplyLatency,
        MeasurementChannel::SaturatedQueueGrowthPerWrite,
    ];
}
