//! Tests pinning the R-1 nearest-rank quantile convention (see [`LatencySummary`] docs): the
//! hand-computed odd-, even-, and 1,000-sample fixtures the spec's Implementation-Time Decision
//! requires. One test entity per file; `quantiles` is the shared nearest-rank helper.

mod even_samples_take_the_lower_middle;
mod from_raw_sorts_before_ranking;
mod full_batch_ramp_summary;
mod odd_sample_quantiles;
mod rank_landing_on_an_integer;

use super::nearest_rank;

/// The R-1 `(Q1, median, Q3, IQR)` of an **already ascending-sorted** slice, exactly as
/// [`LatencySummary::from_raw`](super::LatencySummary::from_raw) reads them after its own sort.
/// Lets the small-n fixtures pin the nearest-rank ranks directly without materializing a full
/// `BATCH_SIZE` [`RawLatencies`](crate::observation::raw_latencies::RawLatencies).
pub(super) fn quantiles(sorted: &[u128]) -> (u128, u128, u128, u128) {
    let q1 = nearest_rank(sorted, 1, 4);
    let median = nearest_rank(sorted, 1, 2);
    let q3 = nearest_rank(sorted, 3, 4);
    (q1, median, q3, q3 - q1)
}
