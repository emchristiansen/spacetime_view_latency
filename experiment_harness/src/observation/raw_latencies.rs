//! The lossless confirmed round-trip latencies of one cumulative dose.

use anyhow::{anyhow, Result};
use serde::{Serialize, Serializer};

use crate::observation::latency_sample::LatencySample;
use crate::params::{BATCH_SIZE, BATCH_SIZE_USIZE};

/// The complete, lossless vector of confirmed round-trip latencies for one dose — exactly one
/// [`LatencySample`] per write in the [`BATCH_SIZE`]-write batch.
///
/// The count is proven *structurally*: the sole constructor [`Self::sealed`] converts into a
/// fixed-size `[LatencySample; BATCH_SIZE_USIZE]`, so after sealing a `RawLatencies` cannot hold any
/// other number of samples — "non-empty" is not merely enforced, exactly `BATCH_SIZE` is the type.
/// That the samples are genuine confirmed round trips (not fabricated) is a separate property,
/// established by the private measurement transition that fills the
/// [`DoseLatencyAccumulator`](super::dose_latency_accumulator::DoseLatencyAccumulator), not by this
/// type.
#[derive(Debug, Clone)]
pub(crate) struct RawLatencies {
    samples: Box<[LatencySample; BATCH_SIZE_USIZE]>,
}

impl RawLatencies {
    /// Seal a full dose's samples, failing loud unless exactly [`BATCH_SIZE`] were collected.
    pub(crate) fn sealed(samples: Vec<LatencySample>) -> Result<Self> {
        let collected = samples.len();
        let samples: Box<[LatencySample; BATCH_SIZE_USIZE]> =
            samples.into_boxed_slice().try_into().map_err(|_| {
                anyhow!(
                    "a dose's raw latency vector must contain exactly {BATCH_SIZE} confirmed \
                     round-trip samples, got {collected}"
                )
            })?;
        Ok(Self { samples })
    }

    /// The lossless samples, always exactly [`BATCH_SIZE`] of them.
    pub(crate) fn samples(&self) -> &[LatencySample; BATCH_SIZE_USIZE] {
        &self.samples
    }
}

impl Serialize for RawLatencies {
    /// Serialized as a flat JSON array of the raw nanosecond samples.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.samples.iter())
    }
}

#[cfg(test)]
mod tests;
