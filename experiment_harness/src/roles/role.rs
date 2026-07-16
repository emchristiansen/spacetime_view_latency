//! The three measurement roles.

/// The three measurement roles, held while subscription count stays constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// M — measured identity with a small, fixed result slice.
    Measured,
    /// M2 — measured identity used only in the own-slice-growth regime.
    OwnSliceMeasured,
    /// G — growth driver, writing keys disjoint from M.
    GrowthDriver,
}
