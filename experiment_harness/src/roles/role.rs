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

impl Role {
    /// A stable canonical tag for this role, for deterministic identity derivation and
    /// machine-readable evidence. An explicit `&'static str` contract rather than
    /// `Debug`/variant-name formatting, so renaming a Rust variant cannot silently change a
    /// derived identity's `from_claims` subject.
    pub fn canonical_tag(self) -> &'static str {
        match self {
            Role::Measured => "measured",
            Role::OwnSliceMeasured => "own-slice-measured",
            Role::GrowthDriver => "growth-driver",
        }
    }
}
