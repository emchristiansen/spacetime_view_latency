//! The frozen method facts carried on every record.

use serde::Serialize;

use crate::control_registry_discovery_screen::screen_params::{SAMPLE_COUNT, WITH_CONFIRMED_READS};
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;

/// The frozen method every record must restate: which channel, how many samples, and whether
/// confirmed reads were in force.
///
/// Carried on *every* record — including a slot that never ran — because the spec's verifiability
/// requirement is that a reader holding only the ledger can confirm what was frozen. A field present
/// on complete records but absent on the rest would force that reader to guess whether an absence
/// meant "false" or "not applicable".
///
/// Written from the frozen constants and never from a caller, so no attempt can report a method it
/// did not run under.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct MethodFacts {
    channel: MeasurementChannel,
    sample_count: u32,
    with_confirmed_reads: bool,
}

impl MethodFacts {
    /// The frozen method for this screen: E3 cold subscription apply, one sample per attempt.
    ///
    /// [`MeasurementChannel`] is reused verbatim rather than restated. The channel vocabulary is
    /// shared, immutable, and already serialized by prior evidence; minting a parallel copy is
    /// exactly the reinterpretation the spec's evidence-immutability rule forbids.
    pub(crate) fn frozen() -> Self {
        Self {
            channel: MeasurementChannel::ColdSubscriptionApplyTime,
            sample_count: SAMPLE_COUNT,
            with_confirmed_reads: WITH_CONFIRMED_READS,
        }
    }
}
