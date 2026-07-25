//! Preregistered parameters specific to the `EntityOwnerSenderView` Pilot (spec c33f2e51).
//!
//! Kept separate from [`crate::params`], which holds the historical campaign's parameters, so this
//! candidate cannot move a historical constant against a recorded seed. Where a historical parameter
//! already expresses the needed invariant it is reused rather than restated.

use crate::params::BATCH_SIZE;

/// The frozen `N_global` ladder each attempt walks progressively.
///
/// The spec's frozen finite range for unrelated/global rows. Frozen means frozen: never extended,
/// truncated, or reordered to chase a result.
pub(crate) const GLOBAL_ROW_LADDER: [u64; 6] = [1_000, 2_000, 4_000, 8_000, 16_000, 32_000];

/// Number of rungs in [`GLOBAL_ROW_LADDER`], for use as an array length bound.
pub(crate) const GLOBAL_ROW_LADDER_LEN: usize = GLOBAL_ROW_LADDER.len();

/// Compile-time proof that the frozen ladder is walkable: strictly ascending, and every rung's
/// increment is at least one measured batch.
///
/// The driver reaches a rung by seeding its increment and measuring the final [`BATCH_SIZE`] writes
/// of that increment, so an increment smaller than a batch would have to measure writes belonging to
/// the next rung. A non-ascending ladder underflows the subtraction here and also fails to compile.
/// Both are properties of the frozen ladder, so they are proven where it is declared rather than
/// re-checked per attempt.
const _: () = {
    let mut rung = 0usize;
    while rung < GLOBAL_ROW_LADDER_LEN {
        let increment = if rung == 0 {
            GLOBAL_ROW_LADDER[0]
        } else {
            GLOBAL_ROW_LADDER[rung] - GLOBAL_ROW_LADDER[rung - 1]
        };
        assert!(
            increment >= BATCH_SIZE,
            "every GLOBAL_ROW_LADDER increment must be at least BATCH_SIZE, so a rung's measured \
             batch fits inside the rows that rung adds"
        );
        rung += 1;
    }
};

/// Exactly this many complete randomized matched blocks in the Pilot stage.
///
/// Distinct from [`crate::params::REPETITION_BLOCKS`] (thirty, the Confirmatory sample): a Pilot
/// that silently borrowed that count would be a different preregistration.
pub(crate) const PILOT_BLOCKS: u32 = 5;

/// [`PILOT_BLOCKS`] as a `usize`, for use as an array length. Guarded by a compile-time round-trip
/// assertion rather than a bare `as` cast, mirroring [`crate::params::BATCH_SIZE_USIZE`], so a
/// platform on which the value does not fit fails to compile instead of truncating.
pub(crate) const PILOT_BLOCKS_USIZE: usize = {
    let as_usize = PILOT_BLOCKS as usize;
    assert!(
        as_usize as u32 == PILOT_BLOCKS,
        "PILOT_BLOCKS does not fit in usize on this platform"
    );
    as_usize
};

/// Fixed literal subject deriving the non-owner global identity, as
/// `quick_run::QUICK_RUN_GROWTH_SUBJECT` does. Each attempt gets a fresh isolated server, so no
/// per-attempt domain separation is needed.
pub(crate) const ENTITY_OWNER_PILOT_GLOBAL_SUBJECT: &str = "entity-owner-pilot-global";

/// Domain-separation label for this Pilot's block-order permutation, distinct from the historical
/// [`crate::params::BLOCK_ORDER_DOMAIN`] so the two cannot collide against the same seed.
pub(crate) const PILOT_BLOCK_ORDER_DOMAIN: &str = "entity-owner-pilot:block-order";

/// Domain-separation label for the per-block Arm/Control order, kept distinct from
/// [`PILOT_BLOCK_ORDER_DOMAIN`] so the role bit is its own derivation rather than a reuse of the
/// block-order key.
pub(crate) const PILOT_ARM_CONTROL_ORDER_DOMAIN: &str = "entity-owner-pilot:arm-control-order";

/// The measured identity's own slice size, held fixed at every rung.
///
/// The Arm's result set is therefore constant while the backing table grows — the read-set behavior
/// under study. Matches [`crate::params::M_SLICE_ROWS`] for comparability, but restated because it
/// is this candidate's preregistered choice.
pub(crate) const OWNED_SLICE_ROWS: u64 = 10;

/// The first `entity_uuid` assigned to the measured identity's rows.
pub(crate) const OWNED_KEY_BASE: u64 = 0;

/// The first `entity_uuid` assigned to the global identity's rows. A billion above
/// [`OWNED_KEY_BASE`] so the two key spaces stay disjoint across the whole ladder, making a
/// primary-key collision structurally impossible — the spacing discipline of
/// [`crate::params::GROWTH_KEY_BASE`].
pub(crate) const GLOBAL_KEY_BASE: u64 = 1_000_000_000;

/// The fixed `record` payload for every seeded and measured row, so payload width never confounds an
/// Arm/Control comparison.
pub(crate) const PILOT_ROW_PAYLOAD: &str = "entity-owner-pilot-fixed-payload";

/// Compile-time proof that the two key spaces cannot collide or overflow.
///
/// The owned slice and the global slice share one `entity_uuid` primary key space, so an overlap
/// would surface as an opaque duplicate-key reducer failure at the first rung rather than as the
/// preregistration error it is. Proven where both bases are declared rather than trusted to the
/// billion-key spacing staying larger than a future owned slice.
const _: () = {
    assert!(
        OWNED_KEY_BASE + OWNED_SLICE_ROWS <= GLOBAL_KEY_BASE,
        "the owned key space must end at or before GLOBAL_KEY_BASE"
    );
    assert!(
        GLOBAL_KEY_BASE.checked_add(GLOBAL_ROW_LADDER[GLOBAL_ROW_LADDER_LEN - 1]).is_some(),
        "the global key space must not overflow u64 at the top of the ladder"
    );
};
