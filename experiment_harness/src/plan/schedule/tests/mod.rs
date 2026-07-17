//! Focused tests for deterministic seeded scheduling. One test entity per file.

mod arm_control_order_is_deterministic_and_varies;
mod covers_every_block_exactly_once;
mod each_block_pairs_its_arm_and_control;
mod order_has_no_duplicate_blocks;
mod same_seed_is_deterministic;
mod selected_seeds_produce_different_orders;
