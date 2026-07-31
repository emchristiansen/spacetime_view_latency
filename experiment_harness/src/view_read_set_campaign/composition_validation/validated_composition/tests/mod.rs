//! Focused tests for the composition check. One test entity per file, over one shared fixture.
//!
//! The fixture builds a *correct* attempt of each role — the transition, the two phase-matched row
//! sets, and a real per-attempt artifact directory — so each failure test states its own defect as a
//! single deviation from it. That keeps every failing case honest: what is being checked is that the
//! named defect is caught, not that some unrelated part of a hand-built input was wrong.

mod a_key_outside_both_ranges_fails_the_census;
mod a_leaked_foreign_row_fails_the_arm;
mod a_missing_owned_row_fails_the_census;
mod a_wrong_owner_fails_the_census;
mod a_wrong_payload_fails_the_census;
mod fixture;
mod neither_the_same_nor_swapped_artifacts_pass;
mod scratch_dir;
mod the_arm_census_accepts_the_required_composition;
mod the_control_census_accepts_the_whole_foreign_slice;
mod the_witness_names_the_final_saturated_write;
