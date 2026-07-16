//! Arms predicted to record key-scoped (point) dependencies.

/// Arms predicted to record key-scoped (point) dependencies.
///
/// These run under **both** growth regimes: flat under other identities' growth,
/// O(own slice) when the measured identity's own slice grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyScopedArm {
    /// F — procedural bounded point-filter over the single-column `sender` btree.
    PointFilter,
    /// F′ — procedural hand-written semijoin: viewer point-filter, then one PK
    /// `.find` per visibility row.
    PointSemijoin,
}
