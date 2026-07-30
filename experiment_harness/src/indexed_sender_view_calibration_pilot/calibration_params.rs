//! Preregistered parameters frozen for the append-only E2 method-calibration pilot (spec
//! c33f2e51).
//!
//! Kept separate from [`crate::params`], [`crate::entity_owner_pilot::pilot_params`], and the
//! discovery screen's own freeze, so this pilot cannot move a constant a recorded campaign, Pilot,
//! or screen already depends on. The one deliberate exception is the rung ladder, which is *read*
//! from the Pilot's frozen [`GLOBAL_ROW_LADDER`] rather than restated: the spec names the existing
//! unrelated/global ladder, and a restated copy would be a second source of truth free to drift.
//!
//! The pinned artifact constants are likewise read or copied from the accepted `ControlRegistry`
//! Step 1 artifacts, because this pilot adds no module change: the indexed arm and its writer
//! reducer are already in that exact WASM.

use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER;

/// Rows the measured identity's own slice is seeded to before any measured append — `S₀`.
///
/// Ten is the governing subscriber-visible baseline, and the spec's decision fixes it explicitly
/// for this pilot. It is deliberately *not* raised to shrink the within-cell growth fraction: that
/// control was proposed and rejected, because bounding a latency effect by a row-count fraction
/// presumes an elasticity from read-set size to latency that this candidate's open mechanism
/// question is precisely about. The growth is disclosed and measured here, never assumed away.
pub(crate) const SUBSCRIBER_OWN_ROWS: u64 = 10;

/// Index of the pilot's single endpoint on [`GLOBAL_ROW_LADDER`] — the 1,000-row baseline rung.
///
/// One rung, not two. This pilot is method calibration: the two attempts are independent
/// replicates, and reading them as a low/high contrast is exactly the comparison the ceiling
/// forbids.
pub(crate) const BASELINE_RUNG_INDEX: usize = 0;

/// Exactly this many independent calibration attempts.
///
/// Two, because one series cannot show between-attempt disagreement and between-attempt
/// disagreement is one of the four things the spec's decision rule weighs. It is not a sample size
/// and supports no aggregate: nothing in this vocabulary combines the two series.
pub(crate) const CALIBRATION_ATTEMPTS: u32 = 2;

/// [`CALIBRATION_ATTEMPTS`] as a `usize`, for use as an array length. Guarded by a compile-time
/// round-trip assertion rather than a bare `as` cast, mirroring [`crate::params::BATCH_SIZE_USIZE`].
pub(crate) const CALIBRATION_ATTEMPTS_USIZE: usize = {
    let as_usize = CALIBRATION_ATTEMPTS as usize;
    assert!(
        as_usize as u32 == CALIBRATION_ATTEMPTS,
        "CALIBRATION_ATTEMPTS does not fit in usize on this platform"
    );
    as_usize
};

/// The frozen, explicit, positive paced sample count every successful replicate records — `W_max`.
///
/// **A ceiling on what an attempt may cost, and simultaneously the exact length of a complete
/// series.** Those are not in tension. `W_max` bounds the run: an attempt may fail before reaching a
/// thousand, and when it does its samples are retained in order as a rejected series rather than
/// fabricated up to the count. What it may not do is count as a replicate — the spec's post-pilot
/// decision rule evaluates every candidate `W` in `{10, 20, 50, 100, 200, 500, 1000}` against *both
/// complete retained series*, and a short series answers neither the 1,000 case nor the
/// between-attempt disagreement the rule weighs.
///
/// Recording a thousand therefore buys the evidence for every candidate count at once, which is why
/// the pilot pays for the longest series it can rather than guessing a shorter one.
pub(crate) const MAX_PACED_SAMPLES: u32 = 1_000;

/// [`MAX_PACED_SAMPLES`] as a `usize`, for capacity and length checks.
pub(crate) const MAX_PACED_SAMPLES_USIZE: usize = {
    let as_usize = MAX_PACED_SAMPLES as usize;
    assert!(
        as_usize as u32 == MAX_PACED_SAMPLES,
        "MAX_PACED_SAMPLES does not fit in usize on this platform"
    );
    as_usize
};

/// Carried explicitly on every inventory and provenance record, as the spec globally requires.
///
/// Not vacuous for this channel, unlike E3's: the pilot issues a real measured write with a real
/// round trip, so confirmed reads genuinely delimit something here. Recorded from a constant anyway
/// so a ledger-only check never has to special-case a channel to find it.
pub(crate) const WITH_CONFIRMED_READS: bool = true;

/// The published distribution the freeze pins every attempt to.
///
/// The upstream release *tag*, recorded alongside the semantic version and release commit because
/// those three are independently checkable facts about one artifact and a reader holding only the
/// ledger should not have to infer any of them from the others.
pub(crate) const DISTRIBUTION_RELEASE_TAG: &str = "v2.7.0-hotfix3";

/// The generated-tree digest accepted at `ControlRegistry` Step 1, reproduced by two independent
/// canonical generation passes.
///
/// The same artifact this pilot measures against, because it adds no module change: the indexed arm
/// `indexed_control_activity_sender_view` and its writer `insert_indexed_control_activity` are both
/// already in that accepted WASM. Written as one canonical lowercase-hex string, transcribed once
/// and verified on construction by [`GeneratedTreeDigest`](super::generated_tree_digest), rather
/// than hand-expanded into a byte array, because a byte array is exactly the shape a silent
/// transcription slip survives in.
pub(crate) const GENERATED_TREE_SHA256_HEX: &str =
    "cc5fdfcb5ca04c835f34a73a65de839fbcfea72b88f0b481768d4c2070e967cf";

/// The exact recipe [`GENERATED_TREE_SHA256_HEX`] was produced by.
///
/// Recorded with the digest because a tree hash is meaningless without its construction rule: a
/// reader reproducing it needs the root, the sort locale, and the manifest-stream order.
pub(crate) const GENERATED_TREE_RECIPE: &str = "from experiment_harness/src/module_artifact, sort \
     all relative file paths under LC_ALL=C, hash each file with SHA-256 in that order, then \
     SHA-256 the resulting manifest stream";

/// The `control_uuid` every row the measured identity owns is attributed to.
///
/// One control, because the synthetic fan-out baseline is one. Production writes 0..=1+G rows per
/// chronicle message, but every row goes to a *distinct* control with its own owner, so from a
/// single subscriber's vantage a production write delivers exactly one visible row whatever `G` is.
/// One control per identity is therefore the faithful baseline rather than a simplification.
pub(crate) const OWN_CONTROL_UUID: u64 = 9_000;

/// The `control_uuid` every unrelated row is attributed to.
///
/// Distinct from [`OWN_CONTROL_UUID`] so the two populations never share a control, which is what
/// makes "these rows are somebody else's" a property of the data rather than of the query.
pub(crate) const UNRELATED_CONTROL_UUID: u64 = 9_500;

/// First activity id in the measured identity's own key range.
///
/// Separated from the unrelated range by a wide gap rather than packed adjacent, so a seeding
/// off-by-one lands in unused space and fails a composition check instead of silently writing a row
/// into the other population. Copies the Pilot's `OWNED_KEY_BASE`/`GLOBAL_KEY_BASE` separation.
pub(crate) const OWN_ACTIVITY_ID_BASE: u64 = 1_000_000_000;

/// First activity id in the unrelated population's key range.
pub(crate) const UNRELATED_ACTIVITY_ID_BASE: u64 = 0;

/// Base timestamp for seeded and appended activity, microseconds since the Unix epoch.
///
/// Fixed rather than clock-derived, so an attempt's seeded state is reproducible from the frozen
/// constants alone — the discipline the empty-view reproducer was corrected to follow.
pub(crate) const TS_BASE_MICROS: i64 = 1_700_000_000_000_000;

/// Microseconds between consecutive activity timestamps, applied to the activity id, so every row
/// in both populations carries a globally unique strictly increasing timestamp.
pub(crate) const TS_STEP_MICROS: i64 = 1_000;

/// The final byte of the non-connecting identity that owns every unrelated row.
///
/// A fixed non-connecting owner, so the unrelated population is provably outside the measured
/// identity's sender-scoped read set without depending on which identity the client happens to
/// connect as. Copies the visible-rows probe's other-owner construction.
pub(crate) const UNRELATED_IDENTITY_BYTE: u8 = 0x40;

/// The greatest activity id any attempt can write.
///
/// The own range sits above the unrelated one and is the longer-lived of the two, so its last
/// append is the extreme: the seeded slice plus every possible append, less one for the zero-based
/// first id. Named rather than recomputed at each use, because it is the bound the timestamp
/// derivation's overflow proof is stated over.
pub(crate) const MAX_ACTIVITY_ID: u64 =
    OWN_ACTIVITY_ID_BASE + SUBSCRIBER_OWN_ROWS + MAX_PACED_SAMPLES as u64 - 1;

/// Compile-time proof that the freeze is self-consistent.
///
/// Every property here is a property of the frozen constants, so it is proven where they are
/// declared rather than re-checked per attempt. Key-space disjointness is the load-bearing one: the
/// measured appends extend the own range by up to `W_max` ids past its `S₀` seeded rows, so the two
/// ranges must stay apart across the *whole* measured run and not merely at seeding time.
const _: () = {
    assert!(
        CALIBRATION_ATTEMPTS >= 2,
        "one series cannot exhibit between-attempt disagreement, which the decision rule weighs"
    );
    assert!(
        MAX_PACED_SAMPLES > 0,
        "the frozen maximum sample count must be explicit and positive"
    );
    assert!(
        SUBSCRIBER_OWN_ROWS > 0,
        "the subscriber must own rows before the first append, or the first sample measures a \
         transition into a non-empty view rather than maintenance of one"
    );
    assert!(
        BASELINE_RUNG_INDEX < GLOBAL_ROW_LADDER.len(),
        "the endpoint must be a position on the existing frozen ladder"
    );
    assert!(
        GLOBAL_ROW_LADDER[BASELINE_RUNG_INDEX] == 1_000,
        "the spec's decision names the 1,000-row baseline global rung"
    );
    assert!(
        OWN_CONTROL_UUID != UNRELATED_CONTROL_UUID,
        "the two populations must not share a control uuid"
    );
    // The own range must hold its seeded slice plus every possible measured append, and must not
    // reach the unrelated range from either direction.
    assert!(
        SUBSCRIBER_OWN_ROWS
            .checked_add(MAX_PACED_SAMPLES as u64)
            .is_some(),
        "the own slice plus every possible append must not overflow u64"
    );
    assert!(
        OWN_ACTIVITY_ID_BASE
            .checked_add(SUBSCRIBER_OWN_ROWS + MAX_PACED_SAMPLES as u64)
            .is_some(),
        "the own key range must not overflow u64"
    );
    assert!(
        UNRELATED_ACTIVITY_ID_BASE + GLOBAL_ROW_LADDER[BASELINE_RUNG_INDEX] <= OWN_ACTIVITY_ID_BASE,
        "the unrelated key range must end before the own range begins"
    );
    // Both populations summed — the witness cache's proven size, which
    // `VerifiedPopulation::verify` adds up and whose `expect` cites this assertion. Stated here
    // rather than left derivable, because an `expect` citing a proof the freeze does not actually
    // contain is the same overclaim this block exists to retire. Bounded entirely by frozen
    // constants once `CalibrationExpectation::after` rejects an append count above the ceiling.
    let own_at_ceiling = match SUBSCRIBER_OWN_ROWS.checked_add(MAX_PACED_SAMPLES as u64) {
        Some(rows) => rows,
        // Unreachable: the assertion above has already failed the build in this case. Written as a
        // match because `Option::expect` is not const-callable.
        None => 0,
    };
    assert!(
        own_at_ceiling
            .checked_add(GLOBAL_ROW_LADDER[BASELINE_RUNG_INDEX])
            .is_some(),
        "the own slice at the append ceiling plus the baseline unrelated population must fit u64"
    );
    assert!(
        TS_STEP_MICROS > 0,
        "timestamps must strictly increase with the activity id"
    );
    // The timestamp derivation is `TS_BASE_MICROS + id × TS_STEP_MICROS` evaluated in `i64`, and it
    // fails loud rather than saturating. These three assertions are what make that `expect`
    // unreachable rather than merely unlikely: the greatest id an attempt can write must convert to
    // `i64`, must survive the multiplication, and must survive the addition. Asserting only that the
    // base plus one step advances proves none of that — it is the smallest offset, not the largest.
    assert!(
        MAX_ACTIVITY_ID <= i64::MAX as u64,
        "the greatest activity id must convert to i64 for the timestamp derivation"
    );
    let scaled = (MAX_ACTIVITY_ID as i64).checked_mul(TS_STEP_MICROS);
    assert!(
        scaled.is_some(),
        "the greatest activity id scaled by the timestamp step must fit i64"
    );
    let scaled = match scaled {
        Some(scaled) => scaled,
        // Unreachable: the assertion above has already failed the build in this case. Written as a
        // match because `Option::expect` is not const-callable.
        None => 0,
    };
    assert!(
        TS_BASE_MICROS.checked_add(scaled).is_some(),
        "the timestamp base plus the greatest offset must fit i64"
    );
};
