//! Resolved SpacetimeDB identities for the measured and growth roles.

use anyhow::{ensure, Result};
use spacetimedb_sdk::Identity;

use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::EXPERIMENT_ISSUER;
use crate::plan::cell::Cell;
use crate::roles::role::Role;

/// The two concrete `spacetimedb_sdk::Identity` values bound to the roles of one run:
/// the active **measured** subscriber and the non-subscribing **growth** driver.
///
/// Constructed only through [`Self::resolve`], which pins the measured identity to the
/// server-issued connection identity and derives the growth identity deterministically,
/// then rejects aliasing. There is no field-wise constructor, so execution code cannot
/// smuggle in ad hoc or colliding identities (spec: "Construct and validate the complete
/// role assignment through the typed `RoleIdentities` boundary rather than accepting ad
/// hoc identities from execution code; measured and growth identities must be distinct").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RoleIdentities {
    measured: Identity,
    growth: Identity,
}

impl RoleIdentities {
    /// Resolve the role assignment for one run.
    ///
    /// `measured` is the **server-issued** identity captured from the measured
    /// subscriber's `on_connect` (2.6.1 view `ctx.sender()` is the authenticated
    /// connection identity; an arbitrary identity is not adoptable by a subscriber
    /// without a server-key-signed JWT — see the Implementation-Time Decision "Align
    /// typed role identities with authenticated subscriber identity"). The growth
    /// identity is derived with [`Identity::from_claims`], domain-separated by the
    /// experiment issuer, schedule seed, cell, and role, and must differ from the
    /// measured identity.
    pub(crate) fn resolve(measured: Identity, seed: ScheduleSeed, cell: Cell) -> Result<Self> {
        let growth = Identity::from_claims(EXPERIMENT_ISSUER, &growth_subject(seed, cell));
        ensure!(
            measured != growth,
            "measured (server-issued) and growth (from_claims) identities collide ({measured:?}); \
             a run's measured and growth identities must be distinct"
        );
        Ok(Self { measured, growth })
    }

    /// The active measured identity (M in `UnrelatedGrowth`, M2 in `OwnSliceGrowth`):
    /// the server-issued subscriber connection identity.
    pub(crate) fn measured(&self) -> Identity {
        self.measured
    }

    /// The non-subscribing growth driver G, writing keys disjoint from the measured
    /// role's.
    pub(crate) fn growth(&self) -> Identity {
        self.growth
    }
}

/// The `from_claims` subject domain-separating the growth identity by schedule seed,
/// cell, and role. The role component is the constant [`Role::GrowthDriver`] tag (only
/// the non-subscribing growth role is derived this way); seed and cell vary it per run.
///
/// Every component is a stable canonical field — the raw seed and the explicit
/// `canonical_tag` contracts — never `Debug` output, so the derived growth identity is
/// reproducible against a recorded seed and cannot be moved by a Rust variant rename.
///
/// `pub(crate)` (spec: "Widen the existing `growth_subject` to `pub(crate)` and reuse it in execution
/// and validation; do not duplicate the derivation grammar") so the analysis validation pass can
/// rederive the same subject from a manifest's already-bound schedule seed and cell, rather than
/// re-encoding this grammar a second time.
pub(crate) fn growth_subject(seed: ScheduleSeed, cell: Cell) -> String {
    format!(
        "seed={};cell={};role={}",
        seed.get(),
        cell.canonical_tag(),
        Role::GrowthDriver.canonical_tag()
    )
}
