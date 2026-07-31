//! Focused tests for the frozen Pilot attempt inventory. One test entity per file; `slot` is the
//! shared fixture helper.

mod each_block_runs_its_arm_and_control_adjacently;
mod frozen_order_matches_independent_derivation;
mod frozen_predeclares_ten_distinct_logical_slots;
mod sealed_rejects_a_repeated_logical_slot;
mod slot;
