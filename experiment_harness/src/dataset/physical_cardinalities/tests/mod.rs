//! Tests for the identity-free physical-cardinality expectation. One test entity per file.
//!
//! The `*_endpoints` tests are the load-bearing proofs: each pins the extracted formula to
//! preregistered constants for one family × regime at the dose-1 and dose-10 ladder endpoints. The
//! `expected_matches_campaign_dataset_*` test is a wiring check that the resolved-dataset path still
//! delegates here; since that delegation is now direct, it cannot prove the formula on its own.

mod chronicle_family_own_slice_growth_endpoints;
mod chronicle_family_unrelated_growth_endpoints;
mod expected_matches_campaign_dataset_across_all_cells_and_doses;
mod message_family_own_slice_growth_endpoints;
mod message_family_unrelated_growth_endpoints;
