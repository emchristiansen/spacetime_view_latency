//! A ladder whose endpoints agree has factor exactly one.

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::endpoint_factor::endpoint_ratio;

use super::fixture;

/// Coverage: the flat case, which the classifier reads against `5/4`, and reduction to canonical
/// form — the quotient arrives as `7/7` and must compare equal to `1/1`.
#[test]
fn a_flat_ladder_has_unit_factor() {
    let cells = fixture::ascending([7, 7, 7, 7, 7, 7]);

    assert_eq!(endpoint_ratio(cells), Rational::from_int(1));
}
