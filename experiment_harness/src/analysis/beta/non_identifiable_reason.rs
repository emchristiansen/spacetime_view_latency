//! The coarse taxonomy of why a block's exponent fit is not an identifiable estimate.

/// The reason a block's secondary fit is not an identifiable estimate (spec: "report `NonIdentifiable`
/// with `NonPositiveScale` or `FlatObjective`"). This is the coarse taxonomy *derived* from a
/// [`BlockFit`](super::block_fit::BlockFit)'s variant (via
/// [`BlockFit::non_identifiable_reason`](super::block_fit::BlockFit::non_identifiable_reason)), not an
/// independently stored field — the authoritative outcome, including any retained candidate, is the
/// `BlockFit` variant itself, so this coarse form cannot disagree with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NonIdentifiableReason {
    /// No exponent in the search domain admits a constrained fit with strictly positive finite scale
    /// `b` — the data carries no positive power-law scale (e.g. an all-zero-latency ladder).
    NonPositiveScale,
    /// A constrained fit exists, but the residual-sum objective is locally flat at the selected interior
    /// exponent: its RSS is not strictly below every *feasible* `β ± 0.001` probe (an out-of-domain probe
    /// is omitted) by more than the relative tolerance, so the exponent is not distinguishable from its
    /// neighbourhood.
    FlatObjective,
}
