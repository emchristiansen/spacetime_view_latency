//! The deterministic seed plan and expected result set for one run.

use crate::dataset::role_slice::RoleSlice;
use crate::dataset::seed_op::SeedOp;
use crate::params::{
    BATCH_SIZE, GROWTH_KEY_BASE, MEASURED_KEY_BASE, M_SLICE_ROWS, OWN_SLICE_BASELINE,
};
use crate::plan::control_table::ControlTable;
use crate::plan::growth_regime::GrowthRegime;
use crate::plan::run::Run;
use crate::roles::role_identities::RoleIdentities;

/// The deterministic dataset for one run: the measured role's slice and the growth
/// driver's slice, over the dataset family (single-table `message` or the Chronicle
/// visibility/message pair) that the run's arm and matched control share.
///
/// This milestone seeds the first rung of the eventual cumulative dose ladder as the
/// representative pre-write state: the measured slice is at its regime-appropriate value
/// (M pinned at [`M_SLICE_ROWS`] under `UnrelatedGrowth`, M2 at one dose of [`BATCH_SIZE`]
/// under `OwnSliceGrowth`) and the growth/unrelated side at its regime-appropriate value
/// (G at one dose of [`BATCH_SIZE`] under `UnrelatedGrowth`, pinned at
/// [`OWN_SLICE_BASELINE`] under `OwnSliceGrowth`). The full ladder and its per-dose growth
/// writes belong to the deferred measurement milestone.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SeedPlan {
    family: ControlTable,
    measured: RoleSlice,
    growth: RoleSlice,
}

impl SeedPlan {
    /// Resolve the deterministic dataset for `run` from its resolved role identities.
    pub(crate) fn resolve(run: Run, identities: &RoleIdentities) -> Self {
        let (measured_count, growth_count) = match run.cell().growth_regime() {
            // M's slice is pinned; G drives N_total. Represent the first growth dose.
            GrowthRegime::UnrelatedGrowth => (M_SLICE_ROWS, BATCH_SIZE),
            // G/unrelated size is pinned; M2 drives N_own. Represent the first M2 dose.
            GrowthRegime::OwnSliceGrowth => (BATCH_SIZE, OWN_SLICE_BASELINE),
        };
        Self {
            family: run.control_table(),
            measured: RoleSlice::new(identities.measured(), MEASURED_KEY_BASE, measured_count),
            growth: RoleSlice::new(identities.growth(), GROWTH_KEY_BASE, growth_count),
        }
    }

    /// The dataset family (single-table `message` vs Chronicle pair) shared by the run's
    /// arm and matched control.
    pub(crate) fn family(&self) -> ControlTable {
        self.family
    }

    /// The measured role's slice (M or M2).
    pub(crate) fn measured(&self) -> RoleSlice {
        self.measured
    }

    /// The growth driver's slice (G).
    pub(crate) fn growth(&self) -> RoleSlice {
        self.growth
    }

    /// The ordered list of confirmed seeding writes that build this dataset on the server.
    pub(crate) fn operations(&self) -> Vec<SeedOp> {
        [self.measured, self.growth]
            .into_iter()
            .flat_map(|slice| {
                let identity = slice.identity();
                slice.keys().map(move |key| match self.family {
                    ControlTable::Message => SeedOp::Message {
                        id: key,
                        sender: identity,
                    },
                    ControlTable::ChronicleMessage => SeedOp::ChroniclePair {
                        key,
                        viewer: identity,
                    },
                })
            })
            .collect()
    }

    /// The number of logical Chronicle visibility/message pairs this plan seeds — the
    /// measured slice plus the growth slice. Only meaningful for the Chronicle family; the
    /// single-table `message` family seeds no visibility pairs. Used to check the live
    /// `message_visibility` read-back's cardinality after seeding
    /// ([`crate::dataset::seeded_visibility::SeededVisibility::assert_unique_pairs`]).
    pub(crate) fn chronicle_pair_count(&self) -> u64 {
        self.measured.count() + self.growth.count()
    }
}
