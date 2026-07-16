//! The preregistered immutable experiment parameters, snapshotted into each manifest.

use serde::Serialize;

use crate::params::{BATCH_DELAY_MS, BATCH_SIZE, CONFIRMED_READS, NUM_DOSES, REPETITION_BLOCKS};

/// The preregistered, immutable experiment parameters recorded in every run manifest so a
/// manifest fully describes the conditions of its run (spec: "Dataset and multi-identity
/// design", "Classification"). Constructed only from the `crate::params` constants, whose
/// values are inherited from Anton's baseline, not inferred from the latency hypotheses.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PreregisteredParameters {
    /// Rows written per dose batch.
    batch_size: u64,
    /// Number of cumulative doses.
    num_doses: u64,
    /// The cumulative dose ladder (`1,000, 2,000, …, 10,000`).
    dose_ladder: Vec<u64>,
    /// Milliseconds between dose batches, outside the measured confirmed round trip.
    batch_delay_ms: u64,
    /// Complete randomized repetition blocks per arm/regime cell.
    repetition_blocks: u32,
    /// Whether confirmed reads are enabled for every primary comparison.
    confirmed_reads: bool,
}

impl PreregisteredParameters {
    /// Snapshot the preregistered parameters from `crate::params`.
    pub(crate) fn preregistered() -> Self {
        let dose_ladder = (1..=NUM_DOSES).map(|dose| dose * BATCH_SIZE).collect();
        Self {
            batch_size: BATCH_SIZE,
            num_doses: NUM_DOSES,
            dose_ladder,
            batch_delay_ms: BATCH_DELAY_MS,
            repetition_blocks: REPETITION_BLOCKS,
            confirmed_reads: CONFIRMED_READS,
        }
    }
}
