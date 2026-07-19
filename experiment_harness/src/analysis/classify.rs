//! The δ-margin primary classification and the independent control-validity gate.
//!
//! Every input is an exact [`Rational`](crate::analysis::stats::rational::Rational) and every interval
//! is a sealed [`MedianCi`](crate::analysis::stats::median_ci::MedianCi); no floating point enters a
//! classification decision. The layer freezes the preregistered practical-equivalence margin, forms the
//! arm's paired-difference and the direct control's total-change intervals over the fixed 30-block
//! sample, and gates each cell:
//!
//! - [`equivalence_margin`] — the frozen `δ = 0.20 · median(L_control)` over the cell's 300 matched
//!   control dose medians, in latency units.
//! - [`response_class`] — the four-way Flat-equivalent/Increasing/Decreasing/Inconclusive class of the
//!   arm's paired-difference interval against `[-δ, +δ]`.
//! - [`prediction_comparison`] — the preregistered [`PredictedResponse`](crate::plan::predicted_response::PredictedResponse)
//!   against the observed [`ResponseClass`](response_class::ResponseClass).
//! - [`cell_evidence`] — the fixed-cardinality per-cell evidence: all 30 arm-minus-control and all 30
//!   direct-control total changes, their schedule-proven collection-order keys, the frozen margin, and
//!   both exact intervals. Retained identically for valid and invalid cells.
//! - [`cell_classification`] — the sum type whose valid branch alone exposes the arm response and
//!   prediction comparison; the `InvalidControl` branch carries the same evidence but no arm claim.
//! - [`classified_campaign`] — the nine per-cell classifications of a validated campaign.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod cell_classification;
pub(crate) mod cell_evidence;
pub(crate) mod classified_campaign;
pub(crate) mod equivalence_margin;
pub(crate) mod prediction_comparison;
pub(crate) mod response_class;
