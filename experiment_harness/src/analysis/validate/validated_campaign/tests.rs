//! Synthetic fixture-driven proofs for the total-validation fold. The shared
//! [`campaign_builder`] constructs a complete, spec-correct campaign programmatically; the success
//! proof folds it, and each category proof applies one minimal mutation and asserts the typed failure.

// `pub(crate)` (test-only, gated by this module's `#[cfg(test)]` parent) so the campaign classifier's
// graph-level tests can reuse this validation-owned fixture without duplicating the complete-campaign
// record builder. The category-proof submodules below stay private.
pub(crate) mod campaign_builder;
mod staged_ladder;

mod a_complete_valid_campaign_folds;

mod a_sequence_gap_outranks_an_earlier_record_defect;
mod an_embedded_disagreement_outranks_its_dangling_resolution;
mod the_census_returns_the_canonically_earliest_failure;

mod a_seed_ordered_campaign_folds_through_the_schedule_order;
mod a_swapped_block_span_fails_the_schedule_order;
mod an_interposed_foreign_block_fails_the_schedule_order;
mod an_opposite_role_order_fails_the_schedule_order;
mod a_manifest_after_its_dose_fails_the_schedule_order;
mod the_earliest_schedule_order_defect_outranks_a_later_one;

mod a_broken_event_identity_is_rejected;
mod a_bumped_summary_median_is_rejected;
mod a_control_coordinate_in_an_arm_position_is_rejected;
mod a_gap_in_the_sequence_is_rejected;
mod a_heterogeneous_nix_store_bin_dir_is_rejected;
mod a_malformed_module_identity_fails_the_binding;
mod a_mismatched_cardinality_ladder_is_rejected;
mod a_missing_cell_fails_the_cell_census;
mod a_non_monotonic_ladder_is_rejected;
mod a_short_latency_vector_fails_the_sample_count;
mod a_wrong_batch_size_is_off_specification;
mod a_wrong_driving_role_tag_fails_coordinate_reference;
mod a_wrong_physical_cardinality_is_rejected;
mod a_zero_server_pid_fails_provenance_shape;
mod an_off_formula_logical_n_is_rejected;
mod an_out_of_range_block_fails_the_block_census;
mod an_out_of_range_dose_fails_the_dose_census;
