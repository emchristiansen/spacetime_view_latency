//! Preregistered parameters of the fresh-server campaign (spec c33f2e51).
//!
//! Kept separate from both [`crate::params`] (the historical campaign's constants) and
//! [`crate::entity_owner_pilot::pilot_params`] (the completed seed-7 cumulative Pilot's), so this
//! protocol cannot move either against evidence already recorded under it. Where a historical
//! parameter already expresses the needed invariant it is reused rather than restated.

use crate::params::{BATCH_DELAY_MS, BATCH_SIZE, BATCH_SIZE_USIZE};

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

/// [`CHANNEL_SAMPLE_COUNT`] as a `usize`, for use as an array length. Reused from
/// [`crate::params::BATCH_SIZE_USIZE`], which already carries the compile-time round-trip proof, so
/// the two cannot disagree about the batch this harness issues.
pub(crate) const CHANNEL_SAMPLE_COUNT_USIZE: usize = BATCH_SIZE_USIZE;

/// Milliseconds between completed paced samples, outside the measured window. Reused from
/// [`crate::params`] for the same reason as [`CHANNEL_SAMPLE_COUNT`].
pub(crate) const PACED_SAMPLE_DELAY_MS: u64 = BATCH_DELAY_MS;

/// Confirmed reads are enabled for every measured channel, and — unlike the completed seed-7
/// ledger — recorded explicitly in this campaign's parameters so a ledger-only check can verify it
/// without inferring it from build provenance.
pub(crate) const CONFIRMED_READS: bool = crate::params::CONFIRMED_READS;

/// How far apart the environment gate's two samples must be, in nanoseconds — the spec's sixty
/// seconds, expressed in the unit the samples' monotonic offsets are recorded in so no conversion
/// happens at the comparison.
pub(crate) const ENVIRONMENT_SAMPLE_SEPARATION_NANOS: u64 = 60 * 1_000_000_000;

/// The minimum available RAM the environment gate admits, in bytes: sixteen gibibytes.
///
/// The spec records why this is 16 GiB and not the operationally-rejected 48: bounded build
/// parallelism and a 32k-row scale point need roughly this much, while swap flow and PSI are what
/// actually guard against pressure.
pub(crate) const ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES: u64 = 16 * 1_024 * 1_024 * 1_024;

/// The maximum memory PSI `full avg60` the environment gate admits, in hundredths — the spec's "at
/// most 1", in the exact integer resolution `/proc/pressure/memory` reports.
pub(crate) const ENVIRONMENT_MAX_MEMORY_PSI_CENTI: u64 = 100;

/// Fixed literal subject deriving the non-owner global identity, as
/// [`crate::entity_owner_pilot::pilot_params::ENTITY_OWNER_PILOT_GLOBAL_SUBJECT`] does for the
/// completed Pilot. Every scale point gets a fresh isolated server, so no per-attempt domain
/// separation is needed. Distinct from that Pilot's literal so the two campaigns' seeded identities
/// are never confused when their ledgers are read side by side.
pub(crate) const GLOBAL_IDENTITY_SUBJECT: &str = "view-read-set-campaign-global";

/// The first `entity_uuid` assigned to the measured identity's own rows.
pub(crate) const OWNED_KEY_BASE: u64 = 0;

/// The first `entity_uuid` assigned to the global identity's rows. A billion above
/// [`OWNED_KEY_BASE`] so the two key spaces stay disjoint at every rung, making a primary-key
/// collision structurally impossible — the spacing discipline of
/// [`crate::params::GROWTH_KEY_BASE`].
pub(crate) const GLOBAL_KEY_BASE: u64 = 1_000_000_000;

/// The one fixed `record` payload every *seeded* row carries, so payload width never confounds a
/// comparison.
///
/// Deliberately named for seeding alone: the measured mutation updates existing primary-key rows
/// with a payload that differs on every write, because the pinned SpacetimeDB source elides a
/// byte-identical update outright. Seeded composition and measured mutation therefore have different
/// payload contracts, and one constant must not appear to cover both.
pub(crate) const SEEDED_ROW_PAYLOAD: &str = "view-read-set-campaign-seeded-payload";

/// The literal prefix of every measured-mutation payload.
///
/// A measured write's payload is `<prefix>:<channel-tag>:<write-index>`, so it differs bytewise from
/// whatever it replaces — which the pinned SpacetimeDB source requires, since a byte-identical
/// update is elided outright and would measure nothing.
pub(crate) const MUTATION_PAYLOAD_PREFIX: &str = "view-read-set-campaign-mutation";

/// The stable payload tag of the paced visible-apply channel's measured writes.
///
/// Tagging by channel is what stops E2 and E1 colliding at their batch boundary: both walk write
/// indices `0..CHANNEL_SAMPLE_COUNT` over the same ten owned keys, so without the tag E1's write `i`
/// could reproduce the payload E2 already left there and be elided as byte-identical.
pub(crate) const PACED_MUTATION_TAG: &str = "e2-paced-visible-apply";

/// The stable payload tag of the saturated channel's measured writes.
pub(crate) const SATURATED_MUTATION_TAG: &str = "e1-saturated-queue-growth";

/// Compile-time proof that a measured batch covers the owned slice a whole number of times.
///
/// The frozen after-state — owned key at offset `k` carrying the payload of write index
/// `CHANNEL_SAMPLE_COUNT - SUBSCRIBER_VISIBLE_ROWS_BASELINE + k` — is only correct when the batch
/// divides evenly by the slice; a remainder would leave the last partial cycle's keys holding
/// payloads from indices the formula does not name. Proven where both constants are visible rather
/// than assumed by the expectation that reads them.
const _: () = {
    assert!(
        CHANNEL_SAMPLE_COUNT % SUBSCRIBER_VISIBLE_ROWS_BASELINE == 0,
        "a measured batch must cover the owned slice a whole number of times, or the frozen \
         final-state formula does not hold"
    );
};

/// Compile-time proof that the two key spaces cannot collide or overflow.
///
/// The owned slice and the global slice share one `entity_uuid` primary key space, so an overlap
/// would surface as an opaque duplicate-key reducer failure rather than as the preregistration error
/// it is. Proven where both bases are declared rather than trusted to the billion-key spacing
/// staying larger than a future owned slice.
///
/// Unlike the completed Pilot's identical proof, the top of the global range is the ladder's largest
/// rung rather than a cumulative total, because no attempt here walks the ladder.
const _: () = {
    assert!(
        OWNED_KEY_BASE + SUBSCRIBER_VISIBLE_ROWS_BASELINE <= GLOBAL_KEY_BASE,
        "the owned key space must end at or before GLOBAL_KEY_BASE"
    );
    assert!(
        GLOBAL_KEY_BASE
            .checked_add(UNRELATED_GLOBAL_ROWS_LADDER[UNRELATED_GLOBAL_ROWS_LADDER_LEN - 1])
            .is_some(),
        "the global key space must not overflow u64 at the top of the ladder"
    );
};
