//! Wiring check: the resolved-`CampaignDataset` path still delegates to the identity-free
//! `PhysicalCardinalities::expected` for every preregistered cell across the whole dose ladder. This
//! guards the delegation edge only; the extracted formula itself is pinned to preregistered constants
//! by the concrete `*_endpoints` tests, since this equality is tautological once the two paths share
//! one implementation.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::cell::Cell;
use crate::roles::role_identities::RoleIdentities;

#[test]
fn expected_matches_campaign_dataset_across_all_cells_and_doses() {
    const SEED: u64 = 20_260_718;

    for cell in Cell::all() {
        // A distinct measured subject per cell keeps it clear of the growth identity; the two differ
        // by issuer regardless, so `resolve` accepts them.
        let measured = Identity::from_claims("test-measured", cell.canonical_tag());
        let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), cell)
            .expect("measured and growth identities must be distinct");
        let dataset = CampaignDataset::resolve(cell.matched_runs()[0], &identities);

        for dose in DoseIndex::ALL {
            let via_dataset = dataset.physical_cardinalities(&DoseBatch::new(&dataset, dose));
            let via_expected = PhysicalCardinalities::expected(cell, dose);
            assert_eq!(
                via_expected,
                via_dataset,
                "identity-free expected cardinalities must equal the resolved-dataset path for \
                 cell {} dose {}",
                cell.canonical_tag(),
                dose.get()
            );
        }
    }
}
