//! Preregistered experiment parameters.
//!
//! The dose ladder is inherited verbatim from `roundtrip_latency_test/src/main.rs`
//! for direct comparability with Anton's baseline (spec: "Dataset and multi-identity
//! design"), not derived from the latency hypotheses. The block count is the fixed
//! preregistered sample from the classification protocol.

/// Rows written per dose batch.
pub(crate) const BATCH_SIZE: u64 = 1_000;

/// Number of cumulative doses (`1,000, 2,000, …, 10,000`).
pub(crate) const NUM_DOSES: u64 = 10;

/// Milliseconds between dose batches, outside the measured confirmed round trip.
pub(crate) const BATCH_DELAY_MS: u64 = 100;

/// Exactly this many complete randomized repetition blocks per arm/regime cell.
/// The distribution-free order-statistic interval assumes this fixed sample; never
/// stop early or extend (spec: "Classification").
pub(crate) const REPETITION_BLOCKS: u32 = 30;

/// Confirmed reads are enabled for every primary comparison to delimit the measured
/// round trip and match Anton's baseline.
pub(crate) const CONFIRMED_READS: bool = true;

/// Expected semantic version of the official CLI and standalone binaries.
pub(crate) const EXPECTED_VERSION: &str = "2.6.1";

/// Expected release commit of the official Nix-packaged 2.6.1 CLI/standalone,
/// matching Anton's retest report (spec: "Official 2.6.1 server provisioning").
pub(crate) const EXPECTED_RELEASE_COMMIT: &str = "052c83fe984a4c4eb7bb4f9afa5c6b1903891d87";
