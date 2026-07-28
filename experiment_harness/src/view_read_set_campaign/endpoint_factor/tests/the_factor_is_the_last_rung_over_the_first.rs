//! The factor is the highest rung's statistic over the lowest rung's, exactly.

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::endpoint_factor::endpoint_ratio;

use super::fixture;

/// Coverage: the rule itself. `10/3` is chosen because it has no exact binary representation, so a
/// float anywhere on this path would fail the equality rather than pass approximately.
#[test]
fn the_factor_is_the_last_rung_over_the_first() {
    let cells = fixture::ascending([3, 4, 5, 6, 8, 10]);

    assert_eq!(endpoint_ratio(cells), Rational::new(10, 3));
}
