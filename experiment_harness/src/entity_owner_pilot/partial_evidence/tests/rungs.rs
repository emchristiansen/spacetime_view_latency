//! Shared fixture: rung evidence at chosen ladder positions.

use std::time::Duration;

use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;
use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE_USIZE;

/// Evidence at each of `positions`, in the order given.
///
/// The latencies are a full sealed batch of distinct samples: sealing partial evidence reads only
/// each element's ladder position, so the sample values are irrelevant here — but they must be a
/// *valid* [`RawLatencies`], since a fixture that could not exist in a real run would prove nothing
/// about one. `positions` indexes [`GlobalRowRung::ALL`], the sole source of rung values, so a test
/// cannot name a rung outside the frozen ladder.
pub(super) fn rungs(positions: &[usize]) -> Vec<RungEvidence> {
    positions
        .iter()
        .map(|position| {
            let samples = (0..BATCH_SIZE_USIZE)
                .map(|i| {
                    let nanos = u64::try_from(i).expect("test sample index fits u64");
                    LatencySample::from_elapsed(Duration::from_nanos(nanos))
                })
                .collect();
            let latencies =
                RawLatencies::sealed(samples).expect("a full batch of latency samples seals");
            RungEvidence::observed(GlobalRowRung::ALL[*position], 0, latencies)
        })
        .collect()
}
