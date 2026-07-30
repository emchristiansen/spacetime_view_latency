//! One contiguous-window median together with the window position it was taken at.

use serde::Serialize;

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;

/// An exact window median paired with the 0-based start position of the window that produced it.
///
/// The position is not decoration. §569 weighs whether a width-`W` median is stable across the
/// series, and "the extreme window median was 3% off" means something different depending on whether
/// that window sat at the very start of the batch — where a cold cache, a warming index, or the
/// first appends after seeding could plausibly explain it — or in the middle of an otherwise settled
/// run. A bare extremum discards exactly the fact that distinguishes those.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct PositionedMedianReport {
    /// The 0-based start index, in the sample series, of the window this median came from.
    position: usize,
    /// That window's exact median.
    median: ExactRationalReport,
}

impl PositionedMedianReport {
    /// Pair one exact window median with its window's start position.
    pub(crate) fn of(position: usize, median: Rational) -> Self {
        Self {
            position,
            median: ExactRationalReport::of(median),
        }
    }
}
