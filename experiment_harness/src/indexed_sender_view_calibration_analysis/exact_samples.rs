//! One replicate's raw samples lifted into exact rational arithmetic.

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;

/// The replicate's series as exact [`Rational`]s, in issue order.
///
/// A raw nanosecond latency is an integer, so this conversion is lossless and every diagnostic built
/// on it — window medians, segment medians, their differences — stays exact. Order is preserved and
/// never sorted here; `median` sorts a copy of whatever slice it is handed, so position information
/// survives for the caller that needs it.
///
/// The `u128 → i128` narrowing is checked rather than cast, and the domain of this exact report is
/// **exactly what `i128` can represent** — no narrower bound is claimed here, and none is derived
/// from the measurement method. A sample that does not fit is outside what these diagnostics can
/// state exactly, and the honest response is to fail loudly: a silent wrap would turn such a value
/// into a plausible-looking *negative* latency and carry it into every median downstream.
pub(crate) fn exact_samples(replicate: &CompleteReplicate) -> Vec<Rational> {
    replicate
        .samples()
        .iter()
        .map(|sample| {
            Rational::from_int(i128::try_from(*sample).expect(
                "this sample is outside i128, so it is outside the domain these exact diagnostics \
                 can state; wrapping would yield a plausible-looking negative latency",
            ))
        })
        .collect()
}
