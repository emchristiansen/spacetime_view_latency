//! The typed carrier of one dose's per-dose outputs, awaiting the future measured-write transition.

use crate::observation::event_evidence::EventEvidence;
use crate::observation::raw_latencies::RawLatencies;

/// The typed carrier of one dose's per-dose outputs — the lossless latency vector and the SDK
/// logical delivery-event evidence — expected from the future measured-write transition. It is the
/// non-serializable input to
/// [`DoseObservation::assemble`](super::dose_observation::DoseObservation::assemble); the derived,
/// serialized record is the observation itself.
///
/// The type and constructor enforce that both parts are *present and well-shaped* (a full
/// [`RawLatencies`] and a checked [`EventEvidence`]); they do not, and cannot, establish that the
/// values came from a real confirmed measurement — that provenance belongs to the measurement
/// transition that constructs this carrier, not to the carrier itself. It is crate-callable.
pub(crate) struct DoseEvidence {
    latencies: RawLatencies,
    events: EventEvidence,
}

impl DoseEvidence {
    pub(crate) fn new(latencies: RawLatencies, events: EventEvidence) -> Self {
        Self { latencies, events }
    }

    /// Split into the raw latencies and event evidence for record assembly.
    pub(crate) fn into_parts(self) -> (RawLatencies, EventEvidence) {
        (self.latencies, self.events)
    }
}
