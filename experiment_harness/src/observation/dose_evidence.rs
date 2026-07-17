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

    /// A deterministic, internally coherent test fixture: a full [`RawLatencies`] of all-1 ns samples and
    /// a checked [`EventEvidence`] whose insert/delete/update counts satisfy the net-delta identity
    /// (`BATCH_SIZE` inserts, net `+BATCH_SIZE`). Mirrors the latency/event construction
    /// [`DoseObservation::fixture_for`](super::dose_observation::DoseObservation) uses; it lets a
    /// no-I/O run-cursor test supply the measured evidence a real dose would return, so
    /// [`RunAwaitingDose::write_observation`](crate::campaign::run_cursor::RunAwaitingDose) can assemble
    /// the observation from its *own* owned context and drawn dose. It is coherent evidence, not an
    /// injected coordinate/dose: the carrier bears no run identity, so it cannot foreign-supply one.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        use std::time::Duration;

        use crate::observation::latency_sample::LatencySample;
        use crate::params::{BATCH_SIZE, BATCH_SIZE_USIZE};

        let samples = vec![LatencySample::from_elapsed(Duration::from_nanos(1)); BATCH_SIZE_USIZE];
        let latencies =
            RawLatencies::sealed(samples).expect("the fixture supplies exactly BATCH_SIZE samples");
        let net_delta = i64::try_from(BATCH_SIZE).expect("BATCH_SIZE fits i64");
        let events = EventEvidence::checked(BATCH_SIZE, 0, 0, net_delta)
            .expect("the fixture event counts satisfy the net-delta identity");
        Self { latencies, events }
    }
}
