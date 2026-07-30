//! The exact composition facts a complete replicate's proof must state.

use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, SUBSCRIBER_OWN_ROWS,
};
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;

/// The three population counts a *complete* attempt's composition proof must carry, all derived from
/// the pilot's own frozen constants rather than restated.
///
/// **Re-admission must reconstruct the whole predicate, not a subset.** The pilot's
/// [`VerifiedPopulation`](crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation)
/// is a capability token minted only by the row-by-row verifier — and that token does not survive
/// serialization. What reaches the ledger is three plain integers. Checking only `verified_appends`
/// would therefore admit a record whose arm or witness cache was proven at some other size, which is
/// precisely the same-cardinality-substitution class the token exists to exclude, re-opened at the
/// read boundary.
///
/// The derivations follow the writer exactly. `VerifiedPopulation::verify` sets `arm_rows` from the
/// own population, which `CalibrationExpectation::after` builds as `S₀ + recorded_appends`;
/// `witness_rows` as that plus the rung's unrelated rows; and `verified_appends` as the recorded
/// count. For a complete replicate the recorded count is the frozen ceiling, which fixes all three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FrozenPopulation {
    /// `S₀ + W_max` — the own slice once every frozen append has landed.
    pub(crate) arm_rows: u64,
    /// The own slice plus the baseline rung's unrelated backing rows.
    pub(crate) witness_rows: u64,
    /// `W_max` — every frozen append, proven landed.
    pub(crate) verified_appends: u64,
}

impl FrozenPopulation {
    /// The composition a complete replicate must have been verified against.
    pub(crate) fn complete() -> Self {
        let verified_appends = u64::from(MAX_PACED_SAMPLES);
        let arm_rows = SUBSCRIBER_OWN_ROWS
            .checked_add(verified_appends)
            .expect("the compile-time freeze proves the seeded slice plus every append fits u64");
        let witness_rows = arm_rows
            .checked_add(CalibrationRung::Baseline.unrelated_rows())
            .expect("the compile-time freeze proves both populations together fit u64");
        Self {
            arm_rows,
            witness_rows,
            verified_appends,
        }
    }
}
