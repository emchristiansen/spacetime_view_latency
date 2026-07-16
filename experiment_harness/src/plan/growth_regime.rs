//! Growth regime — the outer experimental axis.

/// The two mutually exclusive growth regimes.
///
/// Never grow unrelated total size and the measured identity's own slice on the same
/// dose axis (spec: "Dataset and multi-identity design"). Each regime has a separate
/// run and a separate x-axis; which one applies is fixed by the
/// [`crate::plan::cell::Cell`] variant, not chosen freely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrowthRegime {
    /// M's result slice is pinned; growth driver G alone drives `N_total`; the
    /// x-axis is `N_total`. Applies to all seven arms.
    UnrelatedGrowth,
    /// G and unrelated table size are pinned; M2 alone drives `N_own`; the x-axis is
    /// M2's own result-slice size. Reachable only by the key-scoped arms F/F′.
    OwnSliceGrowth,
}
