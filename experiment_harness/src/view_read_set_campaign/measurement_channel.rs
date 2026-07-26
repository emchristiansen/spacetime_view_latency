//! Which of the four measured channels a cell statistic belongs to.

use serde::Serialize;

use crate::view_read_set_campaign::campaign_params::{PACED_MUTATION_TAG, SATURATED_MUTATION_TAG};

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

    /// This channel's stable payload tag, or `None` when it issues no measured writes.
    ///
    /// `None` is the honest answer for the two apply channels rather than an omission: E3 measures a
    /// cold subscription and E4 a reconnect, and neither writes anything, so there is no payload for
    /// them to tag. Returning an `Option` from a total match means a caller must confront that
    /// instead of receiving a tag that names writes the channel never issues.
    ///
    /// The two tags differ so E2's and E1's batches cannot collide: both walk the same write indices
    /// over the same ten owned keys, and an untagged payload from E1's write `i` would reproduce
    /// E2's byte-for-byte — which the pinned source elides, measuring nothing.
    pub(crate) fn mutation_tag(self) -> Option<&'static str> {
        match self {
            Self::PacedVisibleApplyLatency => Some(PACED_MUTATION_TAG),
            Self::SaturatedQueueGrowthPerWrite => Some(SATURATED_MUTATION_TAG),
            Self::ColdSubscriptionApplyTime | Self::ReconnectApplyTime => None,
        }
    }
}
