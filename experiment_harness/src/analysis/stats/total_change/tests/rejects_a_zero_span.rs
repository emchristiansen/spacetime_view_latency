//! A zero span (`n_max == n_min`) is not a degenerate flat change to fold silently; it fails loud.

use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::total_change::total_change;

#[test]
#[should_panic(expected = "strictly increasing ladder span")]
fn rejects_a_zero_span() {
    let _ = total_change(Rational::from_int(2), 5_000, 5_000);
}
