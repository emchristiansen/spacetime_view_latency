//! One preregistered row population, described exactly enough to check every row against it.

use serde::Serialize;
use spacetimedb_sdk::Identity;

use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    TS_BASE_MICROS, TS_STEP_MICROS,
};

/// A contiguous preregistered key range together with every column value its rows must carry.
///
/// **This is what makes composition checkable row by row rather than by count.** The Pilot's
/// `verify_result_set` establishes the argument this copies: because `id` is the table's primary
/// key, distinct rows have distinct ids, so range membership plus an exact per-range count pins the
/// exact expected key set *without materializing it* — while the per-row column checks are what stop
/// a same-cardinality substitution passing. A count-only check accepts the right number of wrong
/// rows; this does not.
///
/// The identity is carried as a value rather than a rule because one of the two populations is owned
/// by whichever identity the client connected as, which is not knowable until runtime.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct ExpectedPopulation {
    /// Stable name for this population in mismatch diagnostics.
    tag: &'static str,
    /// First activity id in the range.
    first_id: u64,
    /// How many rows the range holds — so the range is `first_id .. first_id + rows`.
    rows: u64,
    /// The control every row in this range is attributed to.
    control_uuid: u64,
    /// The identity that owns every row in this range.
    #[serde(
        serialize_with = "crate::indexed_sender_view_calibration_pilot::expected_population::serialize_identity"
    )]
    owner: Identity,
}

/// Render an identity as its canonical hex, so a ledger line names the owner a reader can compare
/// against a provenance record rather than an opaque byte array.
fn serialize_identity<S: serde::Serializer>(
    identity: &Identity,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&identity.to_hex().to_string())
}

impl ExpectedPopulation {
    /// Describe one preregistered population.
    pub(crate) fn new(
        tag: &'static str,
        first_id: u64,
        rows: u64,
        control_uuid: u64,
        owner: Identity,
    ) -> Self {
        Self {
            tag,
            first_id,
            rows,
            control_uuid,
            owner,
        }
    }

    /// This population's name in diagnostics.
    pub(crate) fn tag(self) -> &'static str {
        self.tag
    }

    /// How many rows this population must contain.
    pub(crate) fn rows(self) -> u64 {
        self.rows
    }

    /// Whether `id` falls in this population's preregistered key range.
    ///
    /// Half-open, so two adjacent populations can never both claim an id and a row therefore belongs
    /// to at most one population by construction.
    pub(crate) fn contains(self, id: u64) -> bool {
        let end = self
            .first_id
            .checked_add(self.rows)
            .expect("the compile-time freeze proves every key range fits u64");
        id >= self.first_id && id < end
    }

    /// Every way a row can fail to belong to this population, as diagnostic sentences.
    ///
    /// Returns all of them rather than the first, because "wrong owner *and* wrong control" and
    /// "wrong owner alone" are different faults and the ledger should distinguish them.
    ///
    /// The timestamp check is exact equality against the frozen arithmetic derivation, not a range:
    /// `ts = TS_BASE_MICROS + id × TS_STEP_MICROS` is the whole recipe, so a row whose timestamp does
    /// not reproduce it was not written by the recipe this attempt froze.
    pub(crate) fn row_faults(self, row: &super::observed_row::ObservedRow) -> Vec<String> {
        let mut faults = Vec::new();
        if row.user_identity != self.owner {
            faults.push(format!(
                "{}: row {} is owned by {} rather than {}",
                self.tag,
                row.id,
                row.user_identity.to_hex(),
                self.owner.to_hex(),
            ));
        }
        if row.control_uuid != self.control_uuid {
            faults.push(format!(
                "{}: row {} carries control_uuid {} rather than {}",
                self.tag, row.id, row.control_uuid, self.control_uuid,
            ));
        }
        let expected_ts = expected_ts_micros(row.id);
        if row.ts_micros != expected_ts {
            faults.push(format!(
                "{}: row {} carries ts {}µs rather than the frozen recipe's {}µs",
                self.tag, row.id, row.ts_micros, expected_ts,
            ));
        }
        faults
    }
}

/// The timestamp the frozen recipe assigns to activity `id`.
///
/// Strictly increasing in `id` across both populations, so every row in an attempt carries a
/// globally unique timestamp and a row's id and timestamp cannot disagree about which row it is.
pub(crate) fn expected_ts_micros(id: u64) -> i64 {
    let offset = i64::try_from(id)
        .expect("the compile-time freeze proves every activity id up to MAX_ACTIVITY_ID fits i64");
    let scaled = offset
        .checked_mul(TS_STEP_MICROS)
        .expect("the compile-time freeze proves MAX_ACTIVITY_ID scaled by the step fits i64");
    TS_BASE_MICROS
        .checked_add(scaled)
        .expect("the compile-time freeze proves the base plus the greatest offset fits i64")
}
