//! Shared construction: the frozen rungs, and a ladder of cells over them.
//!
//! [`ScalePoint::ladder`] is the call the frozen inventory itself makes, so these rungs are the
//! preregistered ones rather than a stand-in.

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::axis_ladder::LadderRungIndex;
use crate::view_read_set_campaign::campaign_params::UNRELATED_GLOBAL_ROWS_LADDER_LEN;
use crate::view_read_set_campaign::cell_statistic::CellStatistic;
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::scale_point::ScalePoint;

/// Every rung of the frozen unrelated-global-rows ladder, ascending.
pub(super) fn rungs() -> [LadderRungIndex; UNRELATED_GLOBAL_ROWS_LADDER_LEN] {
    let ladder = ScalePoint::ladder(ExperimentAxis::UnrelatedGlobalRows);
    let rungs: Vec<_> = ladder.iter().map(|scale| scale.rung()).collect();
    match <[LadderRungIndex; UNRELATED_GLOBAL_ROWS_LADDER_LEN]>::try_from(rungs.as_slice()) {
        Ok(rungs) => rungs,
        Err(_) => panic!(
            "the frozen unrelated-global-rows ladder has {UNRELATED_GLOBAL_ROWS_LADDER_LEN} rungs, \
             got {}",
            rungs.len()
        ),
    }
}

/// One statistic, minted through the real validator so a case cannot smuggle in a nonpositive `S`.
pub(super) fn statistic(nanoseconds: i128) -> CellStatistic {
    match CellStatistic::validated(Rational::from_int(nanoseconds)) {
        Ok(statistic) => statistic,
        Err(error) => panic!("{nanoseconds} is a positive cell statistic: {error}"),
    }
}

/// The frozen rungs ascending, paired with the given statistics in that same order.
pub(super) fn ascending(
    nanoseconds: [i128; UNRELATED_GLOBAL_ROWS_LADDER_LEN],
) -> [(LadderRungIndex, CellStatistic); UNRELATED_GLOBAL_ROWS_LADDER_LEN] {
    let rungs = rungs();
    std::array::from_fn(|position| (rungs[position], statistic(nanoseconds[position])))
}
