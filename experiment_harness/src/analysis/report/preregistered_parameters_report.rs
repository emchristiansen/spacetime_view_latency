//! The preregistered-parameters projection embedded in the environment report.

use serde::Serialize;

use crate::manifest::preregistered_parameters::PreregisteredParameters;
use crate::params::NUM_DOSES_USIZE;

/// The preregistered experiment parameters, projected into the report's environment section (spec:
/// campaign provenance "includes ... preregistered parameters"). Every value is an exact integer — batch
/// sizes, dose counts, the cumulative dose ladder, delay, repetition-block count, and confirmed-read count
/// — so no float and no [`FiniteF64`](crate::analysis::finite_f64::FiniteF64) is involved.
#[derive(Debug, Serialize)]
pub(crate) struct PreregisteredParametersReport {
    /// The per-dose driving batch size.
    batch_size: u64,
    /// The number of doses in each run's cumulative ladder.
    num_doses: u64,
    /// The cumulative `dose * batch_size` logical-row ladder — a fixed [`NUM_DOSES_USIZE`]-length array, so
    /// the ten-dose cardinality is a property of the report type (serde serializes arrays up to length 32).
    dose_ladder: [u64; NUM_DOSES_USIZE],
    /// The inter-batch settle delay in milliseconds.
    batch_delay_ms: u64,
    /// The number of repetition blocks per cell/role.
    repetition_blocks: u32,
    /// The number of confirmed reads per dose measurement.
    confirmed_reads: u32,
}

impl PreregisteredParametersReport {
    /// Project the trusted preregistered parameters into their exact-integer report shape.
    pub(crate) fn of(parameters: &PreregisteredParameters) -> Self {
        let _ = parameters;
        todo!("Phase 2: project the exact preregistered parameter integers")
    }
}
