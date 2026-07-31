//! The untrusted wire form of the measurement channel a record was produced on.

use serde::Deserialize;

/// The wire form of
/// [`MeasurementChannel`](crate::view_read_set_campaign::measurement_channel::MeasurementChannel).
///
/// A local mirror rather than a `Deserialize` added to the shared type: that vocabulary is written by
/// recorded campaign, Pilot, and screen evidence, and giving it a deserializer for this analyzer's
/// convenience would widen a type three other subsystems depend on. All four variants are restated so
/// an unexpected channel is *rejected by name* here rather than silently accepted — the frozen-method
/// check in [`MethodFactsDto`](super::method_facts_dto::MethodFactsDto) then refuses everything but
/// the paced channel.
///
/// The wire spelling is proven: the source enum has unit variants and no serde attribute, so serde's
/// default external tagging renders each as its bare variant name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum MeasurementChannelDto {
    SaturatedQueueGrowthPerWrite,
    PacedVisibleApplyLatency,
    ColdSubscriptionApplyTime,
    ReconnectApplyTime,
}
