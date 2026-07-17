//! The resolved per-run dataset: the warm-up background slice plus the driving dose ladder.
//!
//! [`CampaignDataset::resolve`] is the SOLE way to obtain this dataset. It is a total function of
//! a run's [`Run`] and its [`RoleIdentities`]: it derives the control-table family, the growth
//! regime, and which role is pinned versus driving *internally*, so its signature cannot express a
//! wrong pinned/driving assignment or a family mismatched to the run. There is no raw
//! identity/family constructor anywhere, and the pinned background identity is never exposed, so it
//! can never reach a cumulative-dose write.
//!
//! A [`DoseBatch`](super::dose_batch::DoseBatch) is minted only from a resolved `&CampaignDataset`
//! plus a valid [`DoseIndex`], deriving the driving identity, key base, and family from this
//! dataset — so every dose write is the driving role's, in the run's family, by construction.
//! [`Self::for_each_dose`] sequences the whole ladder; see its contract for exactly what ordering
//! it does and does not guarantee.

use anyhow::Result;
use spacetimedb_sdk::Identity;

use crate::dataset::chronicle_physical_rows::ChroniclePhysicalRows;
use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::message_physical_rows::MessagePhysicalRows;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::dataset::seed_op::SeedOp;
use crate::params::{GROWTH_KEY_BASE, MEASURED_KEY_BASE, M_SLICE_ROWS, OWN_SLICE_BASELINE};
use crate::plan::control_table::ControlTable;
use crate::plan::growth_regime::GrowthRegime;
use crate::plan::run::Run;
use crate::roles::role::Role;
use crate::roles::role_identities::RoleIdentities;

/// The deterministic dataset for one measured run: an unmeasured pinned background slice (the
/// warm-up) and the driving role's cumulative dose ladder.
pub(crate) struct CampaignDataset {
    family: ControlTable,
    /// The role whose slice the dose ladder advances (G under `UnrelatedGrowth`, M2 under
    /// `OwnSliceGrowth`), retained for machine-readable evidence.
    driving_role: Role,
    driving: Identity,
    driving_key_base: u64,
    pinned: Identity,
    pinned_key_base: u64,
    pinned_count: u64,
}

impl CampaignDataset {
    /// Resolve the dataset for `run` from its role identities — the sole constructor.
    ///
    /// Total over the regime: `UnrelatedGrowth` pins M and drives with G; `OwnSliceGrowth` pins G
    /// and drives with M2. The family is taken from the run's control table. Because the only
    /// inputs are the whole [`Run`] and [`RoleIdentities`], no caller can misassign the pinned and
    /// driving roles, their key spaces, or the family.
    pub(crate) fn resolve(run: Run, identities: &RoleIdentities) -> Self {
        let family = run.control_table();
        match run.cell().growth_regime() {
            GrowthRegime::UnrelatedGrowth => Self {
                family,
                driving_role: Role::GrowthDriver,
                driving: identities.growth(),
                driving_key_base: GROWTH_KEY_BASE,
                pinned: identities.measured(),
                pinned_key_base: MEASURED_KEY_BASE,
                pinned_count: M_SLICE_ROWS,
            },
            GrowthRegime::OwnSliceGrowth => Self {
                family,
                driving_role: Role::OwnSliceMeasured,
                driving: identities.measured(),
                driving_key_base: MEASURED_KEY_BASE,
                pinned: identities.growth(),
                pinned_key_base: GROWTH_KEY_BASE,
                pinned_count: OWN_SLICE_BASELINE,
            },
        }
    }

    /// The control-table family shared by this run's arm and matched control.
    pub(crate) fn family(&self) -> ControlTable {
        self.family
    }

    /// The driving role's identity — the attribution every cumulative dose write carries. Exposed
    /// only so [`DoseBatch::new`](super::dose_batch::DoseBatch::new) can derive it from a resolved
    /// dataset; it is a value, not a capability to fabricate an out-of-family or pinned write.
    pub(crate) fn driving(&self) -> Identity {
        self.driving
    }

    /// The driving role's contiguous primary-key base, from which each dose's key range advances.
    pub(crate) fn driving_key_base(&self) -> u64 {
        self.driving_key_base
    }

    /// The canonical tag of the driving role, for machine-readable evidence.
    pub(crate) fn driving_role_tag(&self) -> &'static str {
        self.driving_role.canonical_tag()
    }

    /// The physical table cardinalities once `dose` has been applied: the held-constant pinned
    /// background baseline plus this dose's cumulative driving logical rows, projected onto the
    /// family's actual tables. The `message` family occupies a single table (physical == logical);
    /// the Chronicle family occupies two equal tables — one `message_visibility` and one
    /// `chronicle_message` row per logical pair — for a 2× physical footprint. The count is a
    /// property of the resolved dataset (its family and pinned baseline) and the dose, so no
    /// caller can record a family-inconsistent cardinality.
    pub(crate) fn physical_cardinalities(&self, dose: &DoseBatch) -> PhysicalCardinalities {
        // Checked, not assumed: the preregistered constants keep this far below `u64::MAX`, but a
        // checked add fails loud and identically in debug and release rather than silently wrapping
        // if a constant is ever changed out from under this invariant.
        let logical_rows = self
            .pinned_count
            .checked_add(dose.cumulative_driving_rows())
            .expect("dataset logical-row count must not overflow u64");
        match self.family {
            ControlTable::Message => {
                PhysicalCardinalities::Message(MessagePhysicalRows::from_logical_rows(logical_rows))
            }
            ControlTable::ChronicleMessage => PhysicalCardinalities::Chronicle(
                ChroniclePhysicalRows::from_logical_pairs(logical_rows),
            ),
        }
    }

    /// The unmeasured warm-up writes: the pinned background slice, attributed to the pinned
    /// identity, applied through the normal insert reducers before any measured dose. It is never
    /// advanced by the ladder, so it warms the reducer/view path without touching the cumulative
    /// dose x-axis. The pinned identity is confined to this method — it is never handed to a
    /// [`DoseBatch`](super::dose_batch::DoseBatch).
    pub(crate) fn background_operations(&self) -> Vec<SeedOp> {
        (self.pinned_key_base..self.pinned_key_base + self.pinned_count)
            .map(|key| match self.family {
                ControlTable::Message => SeedOp::Message {
                    id: key,
                    sender: self.pinned,
                },
                ControlTable::ChronicleMessage => SeedOp::ChroniclePair {
                    key,
                    viewer: self.pinned,
                },
            })
            .collect()
    }

    /// Deliver every cumulative dose to `f` exactly once, in monotonic `1..=NUM_DOSES` order,
    /// stopping at the first error so no later dose is delivered.
    ///
    /// The guarantee is one of *sequencing and delivery*: because the loop walks the fixed,
    /// module-owned [`DoseIndex::ALL`] and mints each [`DoseBatch`] from `self`, the callback
    /// receives each of the ten doses once, in ladder order, each carrying the driving role's
    /// attribution — and on the first `f` error the ladder truncates immediately rather than
    /// skipping ahead. It does **not** guarantee physical application: the closure still owns
    /// applying each batch's operations exactly once, and nothing here prevents a closure from
    /// ignoring or repeating the operations it is handed.
    pub(crate) fn for_each_dose<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(DoseBatch) -> Result<()>,
    {
        for &dose in &DoseIndex::ALL {
            f(DoseBatch::new(self, dose))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
