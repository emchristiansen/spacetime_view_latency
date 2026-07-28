//! A ladder stored in any order yields the same factor, because rung index decides the endpoints.

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::endpoint_factor::endpoint_ratio;

use super::fixture;

/// Coverage: the sort, and the likeliest defect in practice. Sealing a ladder validates the rung
/// multiset and stores the selections unchanged, so array position carries no order — reading
/// position instead of rung would return the reciprocal here, a plausible positive number nothing
/// downstream would reject.
#[test]
fn rung_order_does_not_change_the_factor() {
    let ascending = fixture::ascending([3, 4, 5, 6, 8, 10]);
    let mut reversed = ascending;
    reversed.reverse();

    assert_eq!(endpoint_ratio(reversed), endpoint_ratio(ascending));
    assert_eq!(endpoint_ratio(reversed), Rational::new(10, 3));
    assert_ne!(
        endpoint_ratio(reversed),
        Rational::new(3, 10),
        "the reciprocal is what a positional read of the reversed ladder would return"
    );
}
