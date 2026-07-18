//! Untrusted mirror of
//! [`PreregisteredParameters`](crate::manifest::preregistered_parameters::PreregisteredParameters).

use serde::Deserialize;

/// The wire form of a manifest's preregistered parameter snapshot. Every field is carried verbatim;
/// `validate` proves the whole campaign's parameters are homogeneous and equal to the frozen
/// `crate::params` constants, so a heterogeneous or off-spec parameter block is rejected there rather
/// than silently trusted here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreregisteredParametersDto {
    pub(crate) batch_size: u64,
    pub(crate) num_doses: u64,
    pub(crate) dose_ladder: Vec<u64>,
    pub(crate) batch_delay_ms: u64,
    pub(crate) repetition_blocks: u32,
    pub(crate) confirmed_reads: bool,
}
