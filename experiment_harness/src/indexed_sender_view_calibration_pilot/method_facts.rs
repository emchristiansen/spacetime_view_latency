//! The frozen method facts carried on every record.

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, SUBSCRIBER_OWN_ROWS, WITH_CONFIRMED_READS,
};
use crate::indexed_sender_view_calibration_pilot::outcome_ceiling::{
    OutcomeCeiling, CALIBRATION_ONLY,
};
use crate::view_read_set_campaign::campaign_params::PACED_SAMPLE_DELAY_MS;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;

/// The frozen method every record must restate: which channel, how many samples, the pacing, the
/// seeded own slice, whether confirmed reads were in force, and the ceiling on what the result may
/// be read as.
///
/// Carried on *every* record — including a slot that never ran — because the spec's verifiability
/// requirement is that a reader holding only the ledger can confirm what was frozen. A field present
/// on recorded attempts but absent on the rest would force that reader to guess whether an absence
/// meant "false" or "not applicable".
///
/// Written from the frozen constants and never from a caller, so no attempt can report a method it
/// did not run under — including the ceiling, which is therefore a fact about the run rather than a
/// claim about it.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct MethodFacts {
    channel: MeasurementChannel,
    sample_count: u32,
    paced_sample_delay_ms: u64,
    seeded_own_rows: u64,
    with_confirmed_reads: bool,
    outcome_ceiling: OutcomeCeiling,
}

impl MethodFacts {
    /// The frozen method for this pilot: E2 paced visible apply, one outstanding append at a time.
    ///
    /// [`MeasurementChannel`] is reused verbatim rather than restated. The channel vocabulary is
    /// shared, immutable, and already serialized by prior evidence; minting a parallel copy is
    /// exactly the reinterpretation the spec's evidence-immutability rule forbids.
    pub(crate) fn frozen() -> Self {
        Self {
            channel: MeasurementChannel::PacedVisibleApplyLatency,
            sample_count: MAX_PACED_SAMPLES,
            paced_sample_delay_ms: PACED_SAMPLE_DELAY_MS,
            seeded_own_rows: SUBSCRIBER_OWN_ROWS,
            with_confirmed_reads: WITH_CONFIRMED_READS,
            outcome_ceiling: CALIBRATION_ONLY,
        }
    }
}
