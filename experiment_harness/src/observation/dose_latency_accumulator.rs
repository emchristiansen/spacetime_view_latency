//! Accumulates one confirmed round-trip sample per measured write into a [`RawLatencies`].

use anyhow::Result;

use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE_USIZE;

/// Collects the confirmed round-trip samples of one dose batch, one per write, then seals them into
/// a [`RawLatencies`].
///
/// This type proves *cardinality* at [`Self::seal`] (exactly `BATCH_SIZE`); it deliberately does
/// **not** claim to prove *provenance*. That every pushed sample is a genuine confirmed round trip
/// is established by the private measurement transition that owns the accumulator and pushes only
/// measured latencies — not by the visibility of [`Self::push`]. The honest boundary: the type
/// bounds the shape, the measurement transition bounds the meaning.
pub(crate) struct DoseLatencyAccumulator {
    samples: Vec<LatencySample>,
}

impl DoseLatencyAccumulator {
    /// A fresh accumulator sized for one full dose batch.
    pub(crate) fn new() -> Self {
        Self {
            samples: Vec::with_capacity(BATCH_SIZE_USIZE),
        }
    }

    /// Record one confirmed write's round-trip latency.
    pub(crate) fn push(&mut self, sample: LatencySample) {
        self.samples.push(sample);
    }

    /// Seal the batch, failing unless exactly `BATCH_SIZE` samples were collected.
    pub(crate) fn seal(self) -> Result<RawLatencies> {
        RawLatencies::sealed(self.samples)
    }
}
