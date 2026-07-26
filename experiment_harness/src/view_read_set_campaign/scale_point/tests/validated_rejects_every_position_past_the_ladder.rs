//! `ScalePoint::validated` is the only fallible way in, and it admits exactly the ladder.

use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::scale_point::ScalePoint;

/// Coverage: the two crate-visible constructors are `ladder` (infallible, ladder-derived) and this
/// one; there is no literal construction, `new`, or setter. So proving this path rejects everything
/// off the ladder, together with the sibling `axis_ladder` tests, closes the range guarantee for the
/// whole exposed surface.
#[test]
fn validated_rejects_every_position_past_the_ladder() {
    let axis = ExperimentAxis::UnrelatedGlobalRows;
    let on_ladder = ScalePoint::ladder(axis).len();

    for position in 0..on_ladder {
        let point = ScalePoint::validated(axis, position)
            .unwrap_or_else(|e| panic!("position {position} is on the ladder: {e:#}"));
        assert_eq!(point.rung().get(), position, "a validated point keeps its rung");
        assert_eq!(point.axis(), axis, "a validated point keeps its axis");
    }

    for position in [on_ladder, on_ladder + 1, 256, usize::MAX] {
        assert!(
            ScalePoint::validated(axis, position).is_err(),
            "position {position} is off the {on_ladder}-rung ladder and must be rejected"
        );
    }
}
