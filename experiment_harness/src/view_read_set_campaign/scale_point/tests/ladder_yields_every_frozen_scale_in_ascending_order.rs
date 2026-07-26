//! `ScalePoint::ladder` names every rung of its axis once, ascending, each reading its own scale.

use crate::view_read_set_campaign::campaign_params::UNRELATED_GLOBAL_ROWS_LADDER;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::scale_point::ScalePoint;

/// Coverage: the inventory's minting path produces exactly the frozen ladder as identities, and each
/// identity resolves its scale through the axis it stored — so a scale point can never report a
/// value from another axis's ladder.
#[test]
fn ladder_yields_every_frozen_scale_in_ascending_order() {
    let points = ScalePoint::ladder(ExperimentAxis::UnrelatedGlobalRows);

    assert_eq!(
        points.len(),
        UNRELATED_GLOBAL_ROWS_LADDER.len(),
        "the ladder must name every frozen rung exactly once"
    );
    for (position, point) in points.iter().enumerate() {
        assert_eq!(point.rung().get(), position, "rungs must ascend without gaps");
        assert_eq!(
            point.axis(),
            ExperimentAxis::UnrelatedGlobalRows,
            "every point must carry the axis it was minted for"
        );
        assert_eq!(
            point.scale(),
            UNRELATED_GLOBAL_ROWS_LADDER[position],
            "rung {position} must resolve its own frozen scale value"
        );
    }
}
