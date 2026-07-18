//! Growth regime — the outer experimental axis.

use crate::params::{M_SLICE_ROWS, OWN_SLICE_BASELINE};
use crate::roles::role::Role;

/// The two mutually exclusive growth regimes.
///
/// Never grow unrelated total size and the measured identity's own slice on the same
/// dose axis (spec: "Dataset and multi-identity design"). Each regime has a separate
/// run and a separate x-axis; which one applies is fixed by the
/// [`crate::plan::cell::Cell`] variant, not chosen freely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum GrowthRegime {
    /// M's result slice is pinned; growth driver G alone drives `N_total`; the
    /// x-axis is `N_total`. Applies to all seven arms.
    UnrelatedGrowth,
    /// G and unrelated table size are pinned; M2 alone drives `N_own`; the x-axis is
    /// M2's own result-slice size. Reachable only by the key-scoped arms F/F′.
    OwnSliceGrowth,
}

impl GrowthRegime {
    /// The held-constant pinned background baseline logical-row count for this regime — the rows the
    /// unmeasured pinned role holds fixed across the entire dose ladder. `UnrelatedGrowth` pins the
    /// measured slice M at [`M_SLICE_ROWS`]; `OwnSliceGrowth` pins the growth driver G at
    /// [`OWN_SLICE_BASELINE`]. This is the single canonical source of the pinned baseline, shared by
    /// dataset resolution ([`CampaignDataset::resolve`](crate::dataset::campaign_dataset::CampaignDataset::resolve))
    /// and the analysis cardinality expectation
    /// ([`PhysicalCardinalities::expected`](crate::dataset::physical_cardinalities::PhysicalCardinalities::expected)),
    /// so the two cannot drift.
    pub(crate) fn pinned_baseline_rows(self) -> u64 {
        match self {
            GrowthRegime::UnrelatedGrowth => M_SLICE_ROWS,
            GrowthRegime::OwnSliceGrowth => OWN_SLICE_BASELINE,
        }
    }

    /// The role whose slice the dose ladder advances in this regime — the growth driver G under
    /// `UnrelatedGrowth`, the own-slice measured role M2 under `OwnSliceGrowth`. This is the single
    /// canonical, identity-free growth-regime→driving-role owner: runtime dataset resolution
    /// ([`CampaignDataset::driving_role`](crate::dataset::campaign_dataset::CampaignDataset)) and the
    /// analysis validation pass (which proves an observation's serialized driving-role tag) both derive
    /// the driving role here, so the two cannot assign the regime a different driving role. It maps only
    /// the regime to the role and requires no run identities, so it can validate the wire tag without
    /// reconstructing a run's identities.
    pub(crate) fn driving_role(self) -> Role {
        match self {
            GrowthRegime::UnrelatedGrowth => Role::GrowthDriver,
            GrowthRegime::OwnSliceGrowth => Role::OwnSliceMeasured,
        }
    }
}
