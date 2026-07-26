//! `rungs` yields every ladder position exactly once, ascending, and no other.

use crate::view_read_set_campaign::axis_ladder::AxisLadder;
use crate::view_read_set_campaign::campaign_params::UNRELATED_GLOBAL_ROWS_LADDER;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;

/// Coverage: the bulk minting path agrees with the frozen literal in count, order, and scale value.
///
/// This is one half of the range guarantee. The other half is structural and has no runtime test to
/// write, because the compiler rejects the code that would test it: `LadderRungIndex`'s field is
/// private to the childless `sealed` module, so nothing outside that module — including *this* test,
/// which is a sibling of it rather than a descendant — can construct one. The only two paths to a
/// rung are `rungs` and `validated`, both inside `sealed`, and both read `len()` first. Verified by
/// compile probe: a `LadderRungIndex(9)` here is `E0423: cannot initialize a tuple struct which
/// contains private fields`.
#[test]
fn rungs_are_exactly_the_frozen_ladder_positions() {
    let ladder = AxisLadder::of(ExperimentAxis::UnrelatedGlobalRows);
    let rungs = ladder.rungs();

    assert_eq!(
        rungs.len(),
        UNRELATED_GLOBAL_ROWS_LADDER.len(),
        "the minted rungs must cover the frozen ladder exactly"
    );
    for (position, rung) in rungs.iter().enumerate() {
        assert_eq!(
            rung.get(),
            position,
            "rungs must be ascending positions with no gap, duplicate, or reordering"
        );
        assert_eq!(
            ladder.scale_at(*rung),
            UNRELATED_GLOBAL_ROWS_LADDER[position],
            "rung {position} must read its own frozen scale value"
        );
    }
}
