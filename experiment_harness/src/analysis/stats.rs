//! The exact reduced-`i128` rational primary estimator.
//!
//! Every value on the primary classification path — each Theil–Sen pairwise slope, the per-block
//! total fitted change, the order-statistic median confidence interval, and the δ equivalence margin
//! — is an exact [`rational::Rational`]. No floating point enters any decision; floats appear only in
//! the human-readable/JSON *display* of these exact values, never in a comparison that classifies a
//! cell.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod median;
pub(crate) mod median_ci;
pub(crate) mod rational;
pub(crate) mod theil_sen;
pub(crate) mod total_change;
