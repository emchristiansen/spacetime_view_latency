//! Preregistered parameters frozen for the `ControlRegistry` discovery E3 screen (spec c33f2e51).
//!
//! Kept separate from [`crate::params`] and [`crate::entity_owner_pilot::pilot_params`] so this
//! screen cannot move a constant a recorded campaign or Pilot seed already depends on. The one
//! exception is deliberate: the rung ladder is *read* from the Pilot's frozen
//! [`GLOBAL_ROW_LADDER`] rather than restated, because the freeze names the existing
//! unrelated/global ladder and a restated copy would be a second source of truth able to drift
//! from it.
//!
//! The seeding constants below — [`CONTROL_UUID_BASE`], [`TS_BASE_MICROS`], [`TS_STEP_MICROS`],
//! [`IDENTITY_BYTE_BASE`] — are the accepted `ControlRegistry` Step 1 recipe copied exactly, per the
//! spec's seeding-implementation decision. Freezing the screen's seeding by reference to the
//! artifact whose capability and semantics were already proven is the point: a second deterministic
//! recipe would be a second thing to get right. An earlier Phase 1 draft named a
//! `SCREEN_CONTROL_SUBJECT` claims-derived identity here; it was never used, and it is removed
//! rather than left contradicting the recipe actually in force.

use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER;

/// Controls held in the registry at every rung — the governing subscriber-visible baseline, `K`.
///
/// Held fixed across both rungs by construction: the screen varies history depth alone, so the
/// registry's own cardinality is the thing that must *not* move.
pub(crate) const REGISTRY_CONTROLS: u64 = 10;

/// Index of the screen's low endpoint on [`GLOBAL_ROW_LADDER`] — 1,000 history rows.
pub(crate) const LOW_RUNG_INDEX: usize = 0;

/// Index of the screen's high endpoint on [`GLOBAL_ROW_LADDER`] — 32,000 history rows.
pub(crate) const HIGH_RUNG_INDEX: usize = 5;

/// Exactly this many independent blocks.
///
/// Two is the counterbalance minimum, not a sample size: one block cannot decouple chronology from
/// rung, because whichever rung it runs first is the one that always runs first. The freeze fixes
/// two and no more, because a screen admits no disposition that a larger sample would sharpen.
pub(crate) const SCREEN_BLOCKS: u32 = 2;

/// [`SCREEN_BLOCKS`] as a `usize`, for use as an array length. Guarded by a compile-time round-trip
/// assertion rather than a bare `as` cast, mirroring [`crate::params::BATCH_SIZE_USIZE`].
pub(crate) const SCREEN_BLOCKS_USIZE: usize = {
    let as_usize = SCREEN_BLOCKS as usize;
    assert!(
        as_usize as u32 == SCREEN_BLOCKS,
        "SCREEN_BLOCKS does not fit in usize on this platform"
    );
    as_usize
};

/// The frozen, explicit, positive sample count.
///
/// One is not a default and not an inheritance: E3's cell statistic *is* the single apply duration,
/// measured exactly once per attempt, because a second subscription on a fresh server is no longer
/// cold. Any other value would describe a different channel. The spec requires this be frozen
/// explicitly so a ledger-only check can verify it, which is why it is a named constant rather than
/// an implicit `1` in the driver.
pub(crate) const SAMPLE_COUNT: u32 = 1;

/// Carried explicitly on every inventory and provenance record, as the spec globally requires.
///
/// Vacuous for this channel — E3 issues no measured write, so there is no round trip for a confirmed
/// read to delimit — but recorded anyway so a ledger-only check never has to special-case a channel
/// to decide whether the field's absence means "false" or "not applicable".
pub(crate) const WITH_CONFIRMED_READS: bool = true;

/// Domain-separation label for the per-`(block, rung)` target order, distinct from every existing
/// order domain so this screen's permutation cannot collide with the campaign's, the Pilot's, or the
/// visible-rows probe's against the same seed.
pub(crate) const SCREEN_TARGET_ORDER_DOMAIN: &str =
    "control-registry-discovery-screen:target-order";

/// The published distribution the freeze pins every attempt to.
///
/// The upstream release *tag*, recorded alongside the semantic version and release commit because
/// those three are independently checkable facts about one artifact and a reader holding only the
/// ledger should not have to infer any of them from the others.
pub(crate) const DISTRIBUTION_RELEASE_TAG: &str = "v2.7.0-hotfix3";

/// The generated-tree digest accepted at `ControlRegistry` Step 1, reproduced by two independent
/// canonical generation passes.
///
/// Written as the spec writes it — one canonical lowercase-hex string, transcribed once and
/// verified on construction by [`GeneratedTreeDigest`](super::generated_tree_digest) — rather than
/// hand-expanded into a byte array, because a byte array is exactly the shape a silent
/// transcription slip survives in. The module WASM hash is *not* restated at all: it is read from
/// the generated [`crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256`] constant the
/// harness already verifies published bytes against, so the ledger and the published bytes cannot
/// disagree.
pub(crate) const GENERATED_TREE_SHA256_HEX: &str =
    "cc5fdfcb5ca04c835f34a73a65de839fbcfea72b88f0b481768d4c2070e967cf";

/// The exact recipe [`GENERATED_TREE_SHA256_HEX`] was produced by.
///
/// Recorded with the digest because a tree hash is meaningless without its construction rule: a
/// reader reproducing it needs the root, the sort locale, and the manifest-stream order, and the
/// Step 1 review rejected an earlier digest precisely for lacking a reproducible recipe.
pub(crate) const GENERATED_TREE_RECIPE: &str = "from experiment_harness/src/module_artifact, sort \
     all relative file paths under LC_ALL=C, hash each file with SHA-256 in that order, then \
     SHA-256 the resulting manifest stream";

/// The first `control_uuid` assigned to the screen's controls. Matches the Step 1 reproducer's base
/// so a reader comparing the two sees the same key space.
pub(crate) const CONTROL_UUID_BASE: u64 = 9_000;

/// Base timestamp for seeded activity, microseconds since the Unix epoch.
///
/// Fixed rather than clock-derived, so a rung's seeded state is reproducible from the frozen
/// constants alone — the discipline the empty-view reproducer was corrected to follow.
pub(crate) const TS_BASE_MICROS: i64 = 1_700_000_000_000_000;

/// Microseconds between consecutive activity timestamps. Every row's `ts` is one step past the
/// previous **global** row's, so all timestamps are globally unique and each control's history is
/// strictly increasing — which the repeat reducer requires and the latest-per-control comparator
/// needs to be unambiguous.
pub(crate) const TS_STEP_MICROS: i64 = 1_000;

/// Base byte of each control's fixed owning identity; control `i` owns `IDENTITY_BYTE_BASE + i`.
///
/// A distinct identity per control rather than one shared subject, so the `control_uuid ->
/// user_identity` dependency the registry preserves is exercised by a column that actually varies
/// across rows.
pub(crate) const IDENTITY_BYTE_BASE: u8 = 0x40;

/// Compile-time proof that the freeze is self-consistent.
///
/// Every property here is a property of the frozen constants, so it is proven where they are
/// declared rather than re-checked per attempt. Divisibility is the load-bearing one: the screen
/// seeds exactly `N / K` rows per control, so a rung whose row count did not divide by `K` would
/// silently truncate and seed a history the freeze does not describe — the same exact-divisibility
/// property Step 1 made compile-time rather than trusting to integer division.
const _: () = {
    assert!(
        SAMPLE_COUNT > 0,
        "the frozen sample count must be explicit and positive"
    );
    assert!(
        SCREEN_BLOCKS >= 2,
        "counterbalancing needs at least two blocks; one block cannot decouple chronology from rung"
    );
    assert!(
        REGISTRY_CONTROLS > 0,
        "K must be positive, or there is no registry to discover"
    );
    assert!(
        LOW_RUNG_INDEX < HIGH_RUNG_INDEX,
        "the low endpoint must precede the high endpoint on the ladder"
    );
    assert!(
        HIGH_RUNG_INDEX < GLOBAL_ROW_LADDER.len(),
        "both endpoints must be positions on the existing frozen ladder"
    );
    assert!(
        GLOBAL_ROW_LADDER[LOW_RUNG_INDEX] == 1_000,
        "the low endpoint the freeze names is 1,000 history rows"
    );
    assert!(
        GLOBAL_ROW_LADDER[HIGH_RUNG_INDEX] == 32_000,
        "the high endpoint the freeze names is 32,000 history rows"
    );
    assert!(
        GLOBAL_ROW_LADDER[LOW_RUNG_INDEX] % REGISTRY_CONTROLS == 0,
        "K must divide the low rung exactly, so every control seeds the same history depth"
    );
    assert!(
        GLOBAL_ROW_LADDER[HIGH_RUNG_INDEX] % REGISTRY_CONTROLS == 0,
        "K must divide the high rung exactly, so every control seeds the same history depth"
    );
    assert!(
        CONTROL_UUID_BASE.checked_add(REGISTRY_CONTROLS).is_some(),
        "the control key space must not overflow u64"
    );
    assert!(
        IDENTITY_BYTE_BASE as u64 + REGISTRY_CONTROLS <= u8::MAX as u64,
        "every control must get a distinct identity byte without wrapping"
    );
    assert!(
        TS_STEP_MICROS > 0,
        "timestamps must strictly increase, or the repeat reducer refuses its own seeding"
    );
};
