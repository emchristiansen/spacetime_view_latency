//! The total fitted change of a block's slope over the primary ladder.

use crate::analysis::stats::rational::Rational;

/// Convert a per-block Theil–Sen `slope` into its total fitted change over the primary ladder,
/// `slope * (n_max - n_min)` (spec: `T_block = slope_block(D) * (N_max - N_min)`). Exact: the slope is
/// a [`Rational`] and the ladder span is an exact integer.
pub(crate) fn total_change(slope: Rational, n_min: i128, n_max: i128) -> Rational {
    // Checked to keep the whole primary path overflow-loud, matching `Rational`'s contract.
    let span = n_max
        .checked_sub(n_min)
        .expect("ladder span N_max - N_min fits i128");
    // The primary ladder is strictly monotonic increasing (spec: the cumulative 1k…10k rungs). A
    // zero or reversed span is a validator/precondition violation, not a value to fold silently into
    // a total change, so encode that required precondition loudly at the stats boundary.
    assert!(
        span > 0,
        "total change requires a strictly increasing ladder span, got N_min={n_min}, N_max={n_max}"
    );
    slope.mul(Rational::from_int(span))
}

#[cfg(test)]
mod tests;
