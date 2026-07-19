//! The exact distribution-free order-statistic confidence interval for a population median.

mod exact_median_ci_30;
mod median_ci;

// The selector has no production caller yet — the classify layer consumes it — so this re-export is
// referenced only by the stats tests in the bin target; keep it surfaced at the module path without a
// spurious unused-import warning until then.
#[allow(unused_imports)]
pub(crate) use exact_median_ci_30::exact_median_ci_30;
pub(crate) use median_ci::MedianCi;

#[cfg(test)]
mod tests;
