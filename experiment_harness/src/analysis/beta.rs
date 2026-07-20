//! The dependency-free secondary `L(N) = a + bN^β` descriptor, computed only downstream of an
//! Increasing primary classification (spec: "Keep the secondary exponent downstream of the exact
//! primary decision").
//!
//! The exponent `β` is inherently real-valued, so — unlike the exact-rational primary classifier — this
//! layer is `f64` throughout. It is nonetheless fully deterministic: a frozen, data-independent bounded
//! one-dimensional search with no optimizer dependency (spec: "Frozen dependency-free fit").
//!
//! - [`block_point`] — one `(N, y)` fit point: an arm dose's logical cardinality and its R-1 median
//!   latency.
//! - [`beta_candidate`] — a fitted `(β, a, b, RSS)` quadruple at one exponent.
//! - [`constrained_fit`] — the constrained least-squares fit over `a ≥ 0, b > 0` at a fixed `β`.
//! - [`non_identifiable_reason`] / [`block_fit`] — the per-block outcome: an identifiable estimate, a
//!   bound-pinned diagnostic, or a preserved non-identifiable failure.
//! - [`fit_block`] — the frozen grid + golden-section search and identifiability test over one block's
//!   ten-dose ladder.
//! - [`beta_label`] / [`beta_population`] — the population `[X_(10), X_(21)]` β interval and its
//!   linear/sublinear consistency label, emitted only when every block is identifiable.
//! - [`beta_descriptor`] — the complete per-cell descriptor: all 30 block outcomes and the optional
//!   population interval.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod beta_candidate;
pub(crate) mod beta_descriptor;
pub(crate) mod beta_label;
pub(crate) mod beta_population;
pub(crate) mod block_fit;
pub(crate) mod block_point;
pub(crate) mod constrained_fit;
pub(crate) mod fit_block;
pub(crate) mod non_identifiable_reason;
