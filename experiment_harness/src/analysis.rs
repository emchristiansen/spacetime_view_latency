//! Preregistered analysis of a complete campaign artifact.
//!
//! A strict, layered pipeline turns an untrusted NDJSON artifact into a machine-readable report:
//!
//! 1. [`ingest`] — untrusted `Deserialize` DTOs mirroring the exact NDJSON wire shape. These are the
//!    crate's *only* `Deserialize` boundary; the production record types are `Serialize`-only and are
//!    never round-tripped, so a malformed artifact cannot masquerade as a trusted value.
//! 2. [`validate`] — total completeness/integrity validation that recomputes and cross-checks every
//!    record (raw sample count, R-1 summary recomputation, deterministic cardinality/event
//!    expectations) and folds the artifact into a trusted campaign graph, or fails loud. A structural
//!    failure here is categorically distinct from a statistically invalid control (the latter is a
//!    typed per-cell outcome, not an ingestion failure).
//! 3. [`stats`] — the exact reduced-`i128` rational primary estimator (Theil–Sen slope, total fitted
//!    change, exact order-statistic median CI): no floating point enters any classification decision.
//! 4. [`classify`] — the δ-margin Flat-equivalent/Increasing/Decreasing/Inconclusive classification
//!    and the independent control-validity equivalence gate.
//! 5. [`beta`] — the dependency-free secondary `L(N)=a+bN^β` descriptor, computed only downstream of an
//!    Increasing classification, under frozen data-independent search constants.
//! 6. [`report`] — the authoritative machine-readable JSON report (stdout is JSON only).
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub(crate) mod classify;
pub(crate) mod ingest;
pub(crate) mod stats;
pub(crate) mod validate;
