//! The fixed-cardinality raw latency vector projection with a manual sequence `Serialize`.

use serde::ser::{Serialize, Serializer};

use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE_USIZE;

/// The report projection of one dose's lossless [`RawLatencies`]: exactly [`BATCH_SIZE_USIZE`] raw
/// nanosecond samples, held in a boxed fixed array so the frozen batch cardinality is a property of the
/// report type rather than a runtime length. The report owns its serialized shape here — it does not defer
/// to the trusted type's own `Serialize`, whose `collect_seq` renders a variable-length sequence that does
/// not carry the cardinality in the type.
///
/// `serde` provides no blanket `Serialize` for arrays longer than 32, and the batch is 1,000 samples, so
/// this newtype implements [`Serialize`] by hand, emitting the array as a JSON sequence of its exact
/// integer nanoseconds — preserving the fixed cardinality without a `Vec` and without a new dependency.
/// Raw nanoseconds stay exact `u128`, never routed through a lossy float boundary.
#[derive(Debug)]
pub(crate) struct RawLatenciesReport(Box<[u128; BATCH_SIZE_USIZE]>);

impl RawLatenciesReport {
    /// Project one dose's lossless raw latency vector. One input — the trusted vector, itself a fixed
    /// [`BATCH_SIZE_USIZE`] array — so the projection's cardinality is proven by its source.
    pub(crate) fn of(latencies: &RawLatencies) -> Self {
        let _ = latencies;
        todo!("Phase 2: copy the BATCH_SIZE_USIZE exact nanosecond samples into the fixed array")
    }
}

impl Serialize for RawLatenciesReport {
    /// Serialize the fixed 1,000-sample array as a plain sequence — a bare JSON array of exact integer
    /// nanoseconds, identical to what a supported-length array would emit.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0.iter())
    }
}
