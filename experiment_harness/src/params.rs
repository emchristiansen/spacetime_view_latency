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

/// The measured identity M's pinned result-slice size in `UnrelatedGrowth` — held
/// fixed while the growth driver G alone drives `N_total` (spec: "Dataset and
/// multi-identity design"; Implementation-Time Decision "Preregister concrete
/// multi-identity cardinalities"). Ten keeps M deliberately small and fixed.
pub(crate) const M_SLICE_ROWS: u64 = 10;

/// The growth driver G's pinned unrelated size in `OwnSliceGrowth` — 1,000 logical
/// rows for message arms or 1,000 logical visibility/message pairs for Chronicle
/// arms, held fixed while M2 alone drives `N_own` (spec: same). One full dose gives a
/// simple nonzero unrelated baseline.
pub(crate) const OWN_SLICE_BASELINE: u64 = 1_000;

/// JWT issuer domain-separating the deterministic non-subscribing growth-role
/// identity derived via `Identity::from_claims` (spec: "Derive the non-subscribing
/// growth role G deterministically with `Identity::from_claims`, domain-separated by
/// the experiment identifier, schedule seed, cell, and role"). Only G uses
/// `from_claims`; the measured role is the server-issued connection identity.
pub(crate) const EXPERIMENT_ISSUER: &str = "view-read-set-experiment";

/// The fixed row payload written for every seeded row of every role. A single
/// constant keeps returned columns and payload width identical across matched
/// arm/control runs and across roles (spec: "Keep returned columns and payload width
/// equivalent across matched arm/control runs"), so payload never confounds a
/// result-set comparison.
pub(crate) const ROW_PAYLOAD: &str = "view-read-set-experiment-fixed-payload";

/// First primary key (`Message.id` / `MessageVisibility.id` / `ChronicleMessage.uuid`)
/// assigned to the measured role's rows. The measured role (M or M2) never coexists
/// with itself across regimes, so it always starts at zero.
pub(crate) const MEASURED_KEY_BASE: u64 = 0;

/// First primary key assigned to the growth driver G's rows. Spaced a full billion
/// keys above the measured base so the two roles' key spaces stay disjoint across the
/// entire dose ladder (max 10,000 keys per role), making a primary-key collision
/// between roles structurally impossible.
pub(crate) const GROWTH_KEY_BASE: u64 = 1_000_000_000;

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
