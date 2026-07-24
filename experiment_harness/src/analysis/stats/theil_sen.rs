//! The exact Theil–Sen slope estimator.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;

/// The Theil–Sen slope of the given points: the median of the exact pairwise slopes over all point
/// pairs with distinct x (equal-x pairs are excluded from the slope set, per the spec's
/// classification protocol). Each pairwise slope `(y_j - y_i) / (x_j - x_i)` is an exact
/// [`Rational`], and the median is taken exactly.
///
/// For the ten-dose ladder every x is distinct, giving `C(10,2) = 45` slopes — an odd count, so the
/// median is a single exact pairwise slope with no averaging.
pub(crate) fn theil_sen_slope(points: &[(i128, i128)]) -> Rational {
    let mut slopes: Vec<Rational> = Vec::new();
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let (x_i, y_i) = points[i];
            let (x_j, y_j) = points[j];
            // Checked, not assumed: the run of `Rational` arithmetic downstream relies on the whole
            // path being overflow-loud (see `Rational`'s type docs). The campaign magnitudes leave
            // vast headroom; a checked subtraction fails identically in debug and release rather than
            // wrapping if that ever ceases to hold.
            let delta_x = x_j.checked_sub(x_i).expect("Theil–Sen Δx fits i128");
            // Exclude equal-x pairs from the pairwise-slope set (spec: classification protocol).
            if delta_x == 0 {
                continue;
            }
            let delta_y = y_j.checked_sub(y_i).expect("Theil–Sen Δy fits i128");
            slopes.push(Rational::new(delta_y, delta_x));
        }
    }
    assert!(
        !slopes.is_empty(),
        "Theil–Sen requires at least one pair of points with distinct x"
    );
    median(&slopes)
}

#[cfg(test)]
mod tests;
