//! The preregistered-parameters projection embedded in the environment report.

use serde::Serialize;

use crate::manifest::preregistered_parameters::PreregisteredParameters;
use crate::params::NUM_DOSES_USIZE;

/// The preregistered experiment parameters, projected into the report's environment section (spec:
/// campaign provenance "includes ... preregistered parameters"). Every value is exact — integer batch
/// sizes, dose counts, the cumulative dose ladder, delay, and repetition-block count, plus the
/// confirmed-reads flag — so no float and no [`FiniteF64`](crate::analysis::finite_f64::FiniteF64) is
/// involved. The projection reproduces the trusted parameters losslessly: `confirmed_reads` is the
/// preregistered boolean flag exactly as the source models it, never a re-encoded count.
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
    /// Whether confirmed reads are enabled for every primary comparison.
    confirmed_reads: bool,
}

impl PreregisteredParametersReport {
    /// Project the trusted preregistered parameters into their exact report shape. The cumulative dose
    /// ladder is the source's `dose * batch_size` sequence, which holds exactly [`NUM_DOSES_USIZE`] entries
    /// by construction, so its collection into the fixed array is total.
    pub(crate) fn of(parameters: &PreregisteredParameters) -> Self {
        let dose_ladder: [u64; NUM_DOSES_USIZE] = parameters
            .dose_ladder()
            .try_into()
            .expect("the preregistered dose ladder holds exactly NUM_DOSES cumulative entries");
        Self {
            batch_size: parameters.batch_size(),
            num_doses: parameters.num_doses(),
            dose_ladder,
            batch_delay_ms: parameters.batch_delay_ms(),
            repetition_blocks: parameters.repetition_blocks(),
            confirmed_reads: parameters.confirmed_reads(),
        }
    }
}

#[cfg(test)]
mod tests;
