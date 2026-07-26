//! Preregistered parameters of the fresh-server campaign (spec c33f2e51).
//!
//! Kept separate from both [`crate::params`] (the historical campaign's constants) and
//! [`crate::entity_owner_pilot::pilot_params`] (the completed seed-7 cumulative Pilot's), so this
//! protocol cannot move either against evidence already recorded under it. Where a historical
//! parameter already expresses the needed invariant it is reused rather than restated.

use crate::params::{BATCH_DELAY_MS, BATCH_SIZE};

/// The frozen campaign seed, from which the rung order of every `(block, role)` is derived.
///
/// Shares its numeric value with the recovery seed by design; the two are separated by their
/// domains ([`RUNG_ORDER_DOMAIN`] versus the recovery domain), which is the same domain-separation
/// discipline the historical Pilot used to drive two independent permutations from seed 7.
pub(crate) const CAMPAIGN_SEED: u64 = 20_260_726;

/// Domain-separation label for the base rung permutation.
///
/// The literal is part of the preregistration: changing it changes every derived execution order,
/// so it is frozen here rather than assembled at a call site.
pub(crate) const RUNG_ORDER_DOMAIN: &str = "view-read-set-experiment:rung-order";

/// Exactly this many complete randomized matched blocks in the Pilot stage.
///
/// Distinct from [`crate::params::REPETITION_BLOCKS`] (thirty, the Confirmatory sample): a stage
/// that silently borrowed another stage's count would be a different preregistration.
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

/// The frozen unrelated/global-rows ladder.
///
/// The spec's frozen finite range for that axis. Unlike the completed seed-7 Pilot's identical
/// literal, these rungs are **not** walked cumulatively on one server: each is a separately
/// provisioned fresh server holding exactly this many rows for the whole measurement, so no
/// increment or per-rung difference is derived from it.
pub(crate) const UNRELATED_GLOBAL_ROWS_LADDER: [u64; 6] = [1_000, 2_000, 4_000, 8_000, 16_000, 32_000];

/// Number of rungs in [`UNRELATED_GLOBAL_ROWS_LADDER`], for use as an array length bound.
pub(crate) const UNRELATED_GLOBAL_ROWS_LADDER_LEN: usize = UNRELATED_GLOBAL_ROWS_LADDER.len();

/// Compile-time proof that the frozen ladder is strictly ascending.
///
/// The endpoint factor `T = S_last / S_first` reads the ladder's two ends, so a ladder that was not
/// ascending would silently invert the estimand's direction. Proven where the ladder is declared
/// rather than re-checked per attempt.
const _: () = {
    let mut rung = 1usize;
    while rung < UNRELATED_GLOBAL_ROWS_LADDER_LEN {
        assert!(
            UNRELATED_GLOBAL_ROWS_LADDER[rung] > UNRELATED_GLOBAL_ROWS_LADDER[rung - 1],
            "UNRELATED_GLOBAL_ROWS_LADDER must be strictly ascending"
        );
        rung += 1;
    }
};

/// The frozen baseline held by the unrelated/global-rows dimension whenever it is *not* the swept
/// axis — the ladder's first rung.
pub(crate) const UNRELATED_GLOBAL_ROWS_BASELINE: u64 = UNRELATED_GLOBAL_ROWS_LADDER[0];

/// The frozen baseline subscriber-visible own slice, held at every rung of a non-visible axis.
///
/// The Arm's result set is therefore constant while the backing table grows — the read-set behavior
/// under study. Matches the completed Pilot's own slice for comparability, but restated because it
/// is this protocol's preregistered choice rather than an inherited one.
pub(crate) const SUBSCRIBER_VISIBLE_ROWS_BASELINE: u64 = 10;

/// The frozen baseline count of active exact keys/subscriptions when that dimension is not the swept
/// axis.
pub(crate) const ACTIVE_EXACT_KEYS_BASELINE: u64 = 1;

/// The frozen baseline synthetic fan-out when that dimension is not the swept axis.
pub(crate) const SYNTHETIC_FAN_OUT_BASELINE: u64 = 1;

/// How many writes the saturated channel issues, and how many paced samples the visible-apply
/// channel takes. Reused from [`crate::params`] rather than restated, so the two protocols cannot
/// disagree about the batch this harness actually issues.
pub(crate) const CHANNEL_SAMPLE_COUNT: u64 = BATCH_SIZE;

/// Milliseconds between completed paced samples, outside the measured window. Reused from
/// [`crate::params`] for the same reason as [`CHANNEL_SAMPLE_COUNT`].
pub(crate) const PACED_SAMPLE_DELAY_MS: u64 = BATCH_DELAY_MS;

/// Confirmed reads are enabled for every measured channel, and — unlike the completed seed-7
/// ledger — recorded explicitly in this campaign's parameters so a ledger-only check can verify it
/// without inferring it from build provenance.
pub(crate) const CONFIRMED_READS: bool = crate::params::CONFIRMED_READS;
