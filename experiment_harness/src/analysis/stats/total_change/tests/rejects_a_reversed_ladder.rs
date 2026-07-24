//! A reversed ladder (`n_max < n_min`) violates the strict-monotonicity precondition and fails loud.

use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::total_change::total_change;

#[test]
#[should_panic(expected = "strictly increasing ladder span")]
fn rejects_a_reversed_ladder() {
    let _ = total_change(Rational::from_int(2), 10_000, 1_000);
}
