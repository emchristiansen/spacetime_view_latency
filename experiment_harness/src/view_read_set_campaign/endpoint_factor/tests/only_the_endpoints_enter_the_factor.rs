//! Intermediate rungs are retained for curves, but never enter `T`.

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::endpoint_factor::endpoint_ratio;

use super::fixture;

/// Coverage: the endpoint clause. The middle four are wildly perturbed and non-monotonic — which
/// the classification permits, asserting nothing about shape between the ends — and the factor is
/// unchanged from the smooth ladder.
#[test]
fn only_the_endpoints_enter_the_factor() {
    let smooth = fixture::ascending([3, 4, 5, 6, 8, 10]);
    let perturbed = fixture::ascending([3, 900, 2, 700, 5, 10]);

    assert_eq!(endpoint_ratio(perturbed), endpoint_ratio(smooth));
    assert_eq!(endpoint_ratio(perturbed), Rational::new(10, 3));
}
