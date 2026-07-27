//! The baseline pair each test varies exactly one reading of.
//!
//! It sits on every threshold it can — separation exactly the frozen interval, RAM exactly the floor,
//! memory pressure exactly the ceiling, swap occupancy nonzero but unchanged — so an inclusive bound
//! written as an exclusive one fails here instead of passing on slack.

use crate::view_read_set_campaign::campaign_params::{
    ENVIRONMENT_MAX_MEMORY_PSI_CENTI, ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
    ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
};
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// A host reporting four CPUs: a load ceiling of 4.00 runnable tasks.
pub(super) const LOGICAL_CPUS: u64 = 4;

pub(super) const FIRST_OFFSET_NANOS: u64 = 0;
pub(super) const SECOND_OFFSET_NANOS: u64 = ENVIRONMENT_SAMPLE_SEPARATION_NANOS;

/// Load falling from 1.00 to 0.50, both well under the ceiling.
pub(super) const FIRST_LOAD_CENTI: u64 = 100;
pub(super) const SECOND_LOAD_CENTI: u64 = 50;

pub(super) const AVAILABLE_RAM_BYTES: u64 = ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES;
/// Nonzero, and identical in both samples: the clause is the delta, not the occupancy.
pub(super) const SWAP_OUT_BYTES: u64 = 8_827_959_175;
pub(super) const PSI_CENTI: u64 = ENVIRONMENT_MAX_MEMORY_PSI_CENTI;

/// The first sample of a pair that passes every clause.
pub(super) fn passing_first() -> EnvironmentSample {
    EnvironmentSample::observed(
        FIRST_OFFSET_NANOS,
        FIRST_LOAD_CENTI,
        AVAILABLE_RAM_BYTES,
        SWAP_OUT_BYTES,
        PSI_CENTI,
    )
}

/// The second sample of that pair.
pub(super) fn passing_second() -> EnvironmentSample {
    EnvironmentSample::observed(
        SECOND_OFFSET_NANOS,
        SECOND_LOAD_CENTI,
        AVAILABLE_RAM_BYTES,
        SWAP_OUT_BYTES,
        PSI_CENTI,
    )
}
