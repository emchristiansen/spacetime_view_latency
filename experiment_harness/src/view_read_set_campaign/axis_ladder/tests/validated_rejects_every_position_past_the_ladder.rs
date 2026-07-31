//! `validated` accepts exactly the in-range positions and rejects everything past the ladder.

use crate::view_read_set_campaign::axis_ladder::AxisLadder;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;

/// Coverage: the fallible minting path — the one reachable with a number from outside the program —
/// admits `0..len` and nothing else, including the off-by-one at `len` itself and a position past
/// `u8::MAX` that would otherwise truncate on the internal narrowing cast.
#[test]
fn validated_rejects_every_position_past_the_ladder() {
    let ladder = AxisLadder::of(ExperimentAxis::UnrelatedGlobalRows);
    let len = ladder.len();

    for position in 0..len {
        let rung = ladder
            .validated(position)
            .unwrap_or_else(|e| panic!("position {position} is on the ladder: {e:#}"));
        assert_eq!(rung.get(), position, "a validated rung keeps its position");
    }

    // `len` is the boundary a `<=` would wrongly admit; 256 is the first value that would wrap to a
    // valid-looking rung if the narrowing cast ran before the range check.
    for position in [len, len + 1, 256, usize::MAX] {
        assert!(
            ladder.validated(position).is_err(),
            "position {position} is off the {len}-rung ladder and must be rejected"
        );
    }
}
