//! The token proving an attempt's rows were validated row by row.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::calibration_expectation::CalibrationExpectation;
use crate::indexed_sender_view_calibration_pilot::composition_mismatch::CompositionMismatch;
use crate::indexed_sender_view_calibration_pilot::expected_population::ExpectedPopulation;
use crate::indexed_sender_view_calibration_pilot::observed_row::ObservedRow;

/// Proof that both caches held exactly their preregistered populations, row by row.
///
/// **A capability token, and the only way to reach a recorded series.**
/// [`CalibrationSeries::recorded`](super::calibration_series::CalibrationSeries::recorded) takes one
/// of these, and [`Self::verify`] is the only function in the crate that can produce one — its
/// fields are private to this module and there is no other constructor. So a series cannot be sealed
/// against an unvalidated population at all: not "must not be", but *cannot be*, because the caller
/// has nothing to pass.
///
/// This replaces an earlier cardinality-only check, which was wrong in the way the Pilot's own
/// `verify_result_set` warns about: equal counts with the wrong owners, the wrong control, a
/// duplicated id substituting for a missing one, or timestamps from a different recipe would all
/// have passed it. For a **sender-scoped** view that is not a small gap — an arm returning the right
/// number of another identity's rows is precisely the leak the candidate must not have.
///
/// The retained counts are the verified ones. They are facts about how much was proven, and carry no
/// duration, so they cannot contribute to a statistic.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct VerifiedPopulation {
    arm_rows: u64,
    witness_rows: u64,
    verified_appends: u64,
}

impl VerifiedPopulation {
    /// Validate both caches against the frozen expectation, row by row.
    ///
    /// Copies [`verify_result_set`](crate::entity_owner_pilot::pilot_driver) in structure and in
    /// argument: **composition, not cardinality.** Every row must fall in one of the preregistered
    /// key ranges and carry that range's exact owner, control, and derived timestamp; the ids in each
    /// range must be distinct; and each range's count must be exact. Because `id` is the table's
    /// primary key, distinctness plus exact count over a range of exactly that size pins the exact
    /// expected key set without materializing it.
    ///
    /// The four checks are run over *all* rows before returning, so the mismatch names every fault
    /// rather than the first — a leak and a lost append discovered together should both reach the
    /// ledger.
    ///
    /// The arm is checked against the own population alone, because a sender-scoped view must return
    /// the measured identity's rows **and only those**: an unrelated row appearing there is a leak,
    /// which is why an out-of-range row in the arm is a fault rather than a row simply belonging to
    /// the other population.
    pub(crate) fn verify(
        expected: CalibrationExpectation,
        arm: &[ObservedRow],
        witness: &[ObservedRow],
    ) -> Result<Self, CompositionMismatch> {
        let own = expected.own_population();
        let unrelated = expected.unrelated_population();
        let mut faults = Vec::new();

        // The arm: the own population and nothing else.
        check_exclusive(&mut faults, "arm", own, arm);

        // The witness: the union of both populations, each checked against its own rules, with no
        // row belonging to neither.
        let mut witness_own = Vec::new();
        let mut witness_unrelated = Vec::new();
        for row in witness {
            if own.contains(row.id) {
                witness_own.push(*row);
            } else if unrelated.contains(row.id) {
                witness_unrelated.push(*row);
            } else {
                faults.push(format!(
                    "witness: row {} falls in neither preregistered key range",
                    row.id
                ));
            }
        }
        check_population(&mut faults, "witness", own, &witness_own);
        check_population(&mut faults, "witness", unrelated, &witness_unrelated);

        // Branch on the fault list directly. An earlier draft minted the token from
        // `CompositionMismatch::of(..)` returning `Err`, which happened to be correct only because
        // "empty" is that constructor's sole rejection today — any validation added to it later
        // would have silently become a *proof of success*. Converting an arbitrary error into a
        // capability token is exactly the shape this module exists to prevent.
        if !faults.is_empty() {
            return Err(CompositionMismatch::of(faults)
                .expect("the list is nonempty here, which is the constructor's only requirement"));
        }

        let arm_rows = expected.own_population().rows();
        Ok(Self {
            arm_rows,
            witness_rows: arm_rows
                .checked_add(expected.unrelated_population().rows())
                .expect("the compile-time freeze proves both populations together fit u64"),
            verified_appends: expected.recorded_appends(),
        })
    }

    /// Rows proven present in the arm cache.
    pub(crate) fn arm_rows(self) -> u64 {
        self.arm_rows
    }

    /// Rows proven present in the witness cache.
    pub(crate) fn witness_rows(self) -> u64 {
        self.witness_rows
    }

    /// Appends this verification proved landed in the subscriber's own slice.
    ///
    /// Read by [`CalibrationSeries::recorded`](super::calibration_series::CalibrationSeries::recorded)
    /// to refuse a complete series whose composition was verified against a different number of
    /// appends, so the sample count, the verified row count, and the frozen ceiling must all agree.
    pub(crate) fn verified_appends(self) -> u64 {
        self.verified_appends
    }
}

/// Check that `rows` is exactly `expected`, and that no row falls outside it.
///
/// Used for the arm, where an out-of-range row is a sender-scope leak rather than a row that belongs
/// somewhere else.
fn check_exclusive(
    faults: &mut Vec<String>,
    cache: &str,
    expected: ExpectedPopulation,
    rows: &[ObservedRow],
) {
    for row in rows {
        if !expected.contains(row.id) {
            faults.push(format!(
                "{cache}: row {} is outside {}'s preregistered key range, so this cache returned a \
                 row the measured identity does not own",
                row.id,
                expected.tag(),
            ));
        }
    }
    check_population(faults, cache, expected, rows);
}

/// Check every column of every row against `expected`, plus id distinctness and exact count.
///
/// Distinctness is checked explicitly rather than assumed from the primary key: the count argument
/// that pins the key set depends on it, and a harness-side snapshot that duplicated a row would
/// otherwise satisfy a count while covering one fewer id.
fn check_population(
    faults: &mut Vec<String>,
    cache: &str,
    expected: ExpectedPopulation,
    rows: &[ObservedRow],
) {
    let mut seen = BTreeSet::new();
    for row in rows {
        if expected.contains(row.id) {
            faults.extend(expected.row_faults(row));
            if !seen.insert(row.id) {
                faults.push(format!(
                    "{cache}: {} holds id {} more than once, so its rows cannot cover the expected \
                     key set",
                    expected.tag(),
                    row.id,
                ));
            }
        }
    }
    let observed = u64::try_from(seen.len()).expect("a cache holds fewer rows than u64 can count");
    if observed != expected.rows() {
        faults.push(format!(
            "{cache}: {} holds {observed} distinct rows rather than the expected {}",
            expected.tag(),
            expected.rows(),
        ));
    }
}
