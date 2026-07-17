//! Focused pure tests for [`SubscribedTable::expected_through_dose`](super::SubscribedTable::expected_through_dose):
//! the cumulative expected-set derivation across both growth regimes and both target scopes.
//!
//! Each file fixes one (growth regime × target scope) combination and pins the expected set at dose 1
//! and dose 10 of the ladder, so full-table cumulative growth and measured-slice selective growth are
//! both proven at the ends of the primary ladder. One test entity per file.

mod a_full_table_target_under_own_slice_growth_grows_with_the_measured_slice;
mod a_full_table_target_under_unrelated_growth_grows_by_one_batch_per_dose;
mod a_measured_slice_target_under_own_slice_growth_grows_with_every_dose;
mod a_measured_slice_target_under_unrelated_growth_stays_pinned;
