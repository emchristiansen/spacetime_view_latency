//! Resolved SpacetimeDB identities for the three measurement roles.

use spacetimedb_sdk::Identity;

/// The concrete `spacetimedb_sdk::Identity` bound to each measurement role in one
/// run. Uses the published 2.6.1 client identity type directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleIdentities {
    measured: Identity,
    own_slice_measured: Identity,
    growth_driver: Identity,
}

impl RoleIdentities {
    /// Bind resolved identities to the three roles.
    pub fn new(measured: Identity, own_slice_measured: Identity, growth_driver: Identity) -> Self {
        Self {
            measured,
            own_slice_measured,
            growth_driver,
        }
    }

    /// M — the measured identity with a small, fixed result slice.
    pub fn measured(&self) -> Identity {
        self.measured
    }

    /// M2 — the measured identity used only in the own-slice-growth regime.
    pub fn own_slice_measured(&self) -> Identity {
        self.own_slice_measured
    }

    /// G — the growth driver writing keys disjoint from M.
    pub fn growth_driver(&self) -> Identity {
        self.growth_driver
    }
}
