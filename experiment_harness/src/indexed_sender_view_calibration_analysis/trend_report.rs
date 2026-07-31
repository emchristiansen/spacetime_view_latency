//! Early-versus-late drift, as two exact named segment medians and their difference.

use serde::Serialize;

use crate::analysis::stats::median::median;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::exact_samples::exact_samples;

/// One replicate's early/late drift, stated as **numbers rather than a verdict**.
///
/// §569 weighs append-series trend, and the honest minimum for that is: what was the median over the
/// first half, what was it over the second, and what is the exact difference. No slope fit, no
/// significance test, no "drifting"/"stationary" label — each of those would embed a modelling
/// assumption or a threshold that §568 gives this pilot no authority to choose.
///
/// **The split is defined here, not assumed.** The series is split at `n / 2` into a first segment
/// `[0, n/2)` and a second `[n/2, n)`; for the frozen thousand-sample series both halves are exactly
/// 500 long. `difference` is `late − early`, so a positive value means the run got *slower* over
/// time — the direction stated explicitly, since a sign convention left implicit is a sign convention
/// a reader will guess wrong.
///
/// Both medians are exact rationals over the raw nanosecond samples, so the difference is exact too.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct TrendReport {
    /// Samples in the first segment.
    early_samples: usize,
    /// Samples in the second segment.
    late_samples: usize,
    /// The exact median of the first segment.
    early_median: ExactRationalReport,
    /// The exact median of the second segment.
    late_median: ExactRationalReport,
    /// The exact `late_median − early_median`. Positive means later samples ran slower.
    difference: ExactRationalReport,
}

impl TrendReport {
    /// Split one replicate's series in half and report both exact segment medians and their exact
    /// difference.
    pub(crate) fn of(replicate: &CompleteReplicate) -> Self {
        let samples = exact_samples(replicate);
        let split = samples.len() / 2;
        assert!(
            split > 0 && split < samples.len(),
            "both segments must be nonempty for their medians to be defined; admission fixes the \
             series at the frozen count, which is far above two"
        );

        let early_median = median(&samples[..split]);
        let late_median = median(&samples[split..]);
        Self {
            early_samples: split,
            late_samples: samples.len() - split,
            early_median: ExactRationalReport::of(early_median),
            late_median: ExactRationalReport::of(late_median),
            // Late minus early, so a positive value means later samples ran slower.
            difference: ExactRationalReport::of(late_median.sub(early_median)),
        }
    }
}

#[cfg(test)]
mod tests;
