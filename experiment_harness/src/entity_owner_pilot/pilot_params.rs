//! Preregistered parameters specific to the `EntityOwnerSenderView` Pilot (spec c33f2e51).
//!
//! These are candidate-specific and deliberately live here rather than in [`crate::params`], which
//! holds the historical Message/Chronicle campaign's preregistered parameters. Keeping them separate
//! means this candidate cannot silently move a historical constant against a recorded seed. Where a
//! historical parameter already expresses the needed invariant it is reused directly rather than
//! restated — see [`crate::params::BATCH_SIZE`] and [`crate::params::REPETITION_BLOCKS`].

/// The frozen `N_global` ladder: the unrelated/global row counts each attempt walks progressively.
///
/// Verbatim the spec's frozen finite range for unrelated/global rows ("unrelated/global rows: `1k,
/// 2k, 4k, 8k, 16k, 32k`") and the Parked Frontier's `N_global`. Frozen means frozen: this array is
/// the preregistration, so it is never extended, truncated, or reordered to chase a result.
pub(crate) const GLOBAL_ROW_LADDER: [u64; 6] = [1_000, 2_000, 4_000, 8_000, 16_000, 32_000];

/// Number of rungs in [`GLOBAL_ROW_LADDER`], for use as an array/iterator length bound.
pub(crate) const GLOBAL_ROW_LADDER_LEN: usize = GLOBAL_ROW_LADDER.len();

/// Exactly this many complete randomized matched blocks in the Pilot stage.
///
/// The spec's "Pilot: five randomized matched blocks". Distinct from
/// [`crate::params::REPETITION_BLOCKS`] (thirty), which is the Confirmatory sample — a Pilot that
/// silently borrowed the Confirmatory count would be a different preregistration.
pub(crate) const PILOT_BLOCKS: u32 = 5;

/// [`PILOT_BLOCKS`] as a `usize`, for use as an array length. Guarded by a compile-time round-trip
/// assertion rather than a bare `as` cast, mirroring [`crate::params::BATCH_SIZE_USIZE`], so a
/// platform on which the value does not fit a `usize` fails to compile instead of silently
/// truncating the array length.
pub(crate) const PILOT_BLOCKS_USIZE: usize = {
    let as_usize = PILOT_BLOCKS as usize;
    assert!(
        as_usize as u32 == PILOT_BLOCKS,
        "PILOT_BLOCKS does not fit in usize on this platform"
    );
    as_usize
};

/// Fixed literal subject deriving the Pilot's distinct non-owner global identity, analogous to
/// `quick_run::QUICK_RUN_GROWTH_SUBJECT` and
/// `entity_owner_smoke::ENTITY_OWNER_SMOKE_OTHER_OWNER_SUBJECT`. Each attempt provisions its own
/// fresh isolated server, so no per-attempt domain separation is needed.
pub(crate) const ENTITY_OWNER_PILOT_GLOBAL_SUBJECT: &str = "entity-owner-pilot-global";

/// Domain-separation label for this Pilot's deterministic block-order permutation, kept distinct
/// from the historical [`crate::params::BLOCK_ORDER_DOMAIN`] so this candidate's seeded ordering
/// can never collide with the historical campaign's against the same seed.
pub(crate) const PILOT_BLOCK_ORDER_DOMAIN: &str = "entity-owner-pilot:block-order";

/// Domain-separation label for the per-block Arm/Control order randomization, kept distinct from
/// [`PILOT_BLOCK_ORDER_DOMAIN`] so the role bit is its own derivation rather than a reuse of the
/// block-order key.
pub(crate) const PILOT_ARM_CONTROL_ORDER_DOMAIN: &str = "entity-owner-pilot:arm-control-order";

/// The measured identity's own pinned slice size, held fixed at every rung while the global
/// identity alone drives `N_global`.
///
/// The Arm's sender-scoped view returns exactly these rows at every rung, so the Arm's *result set*
/// is constant while the *backing table* grows — which is precisely the read-set behavior under
/// study. Matches the historical [`crate::params::M_SLICE_ROWS`] value for comparability, but is
/// restated here because it is this candidate's preregistered choice, not an inherited one.
pub(crate) const OWNED_SLICE_ROWS: u64 = 10;

/// The first `entity_uuid` assigned to the measured identity's owned rows.
pub(crate) const OWNED_KEY_BASE: u64 = 0;

/// The first `entity_uuid` assigned to the non-owner global identity's rows. Spaced a full billion
/// keys above [`OWNED_KEY_BASE`] so the two roles' key spaces stay disjoint across the entire
/// ladder, making a primary-key collision between roles structurally impossible — the same spacing
/// discipline as [`crate::params::GROWTH_KEY_BASE`].
pub(crate) const GLOBAL_KEY_BASE: u64 = 1_000_000_000;

/// The fixed `record` payload written for every seeded and measured row of every role, so payload
/// width never confounds an Arm/Control comparison. Mirrors [`crate::params::ROW_PAYLOAD`]'s intent
/// for this candidate's `entity_owner.record` column.
pub(crate) const PILOT_ROW_PAYLOAD: &str = "entity-owner-pilot-fixed-payload";
