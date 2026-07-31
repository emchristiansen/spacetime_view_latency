//! Each endpoint runs first in exactly one block, at every seed.

use std::collections::BTreeSet;

use crate::control_registry_discovery_screen::attempt_inventory::AttemptInventory;
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::run_role::RunRole;

/// Coverage: the counterbalance the freeze requires actually holds — across the two blocks, each
/// endpoint leads exactly once, so neither rung is confounded with chronological position.
///
/// Asserted at every seed, not on average across seeds, which is precisely why the rung order is
/// derived from block parity rather than from the seed: a seeded coin would fail this a quarter of
/// the time at two blocks, and a screen that had to be re-run until its order came out balanced
/// would not be counterbalanced at all.
///
/// Also checks each block runs both rungs, so "balanced" cannot be achieved by a block silently
/// dropping an endpoint.
#[test]
fn rung_order_is_counterbalanced_across_blocks() {
    const SEEDS: [u64; 4] = [0, 1, 7, 0xDEAD_BEEF];

    for seed in SEEDS {
        let frozen = AttemptInventory::frozen(ScheduleSeed::new(seed)).expect("the seed freezes");

        // The first rung encountered in each block, in block order.
        let mut leaders: Vec<ScreenRung> = Vec::new();
        let mut seen_in_block: BTreeSet<ScreenRung> = BTreeSet::new();
        let mut block_rungs: Vec<BTreeSet<ScreenRung>> = Vec::new();
        let mut current_block: Option<_> = None;

        for attempt in frozen.attempts() {
            let block = attempt.stage_block();
            if current_block != Some(block) {
                if current_block.is_some() {
                    block_rungs.push(std::mem::take(&mut seen_in_block));
                }
                current_block = Some(block);
                leaders.push(attempt.rung());
            }
            seen_in_block.insert(attempt.rung());
        }
        block_rungs.push(seen_in_block);

        assert_eq!(
            leaders.len(),
            2,
            "seed {seed} must run exactly two blocks, got {}",
            leaders.len()
        );
        assert_ne!(
            leaders[0], leaders[1],
            "seed {seed} let the same endpoint lead both blocks, so rung is confounded with order"
        );
        for (block, rungs) in block_rungs.iter().enumerate() {
            assert_eq!(
                rungs.len(),
                2,
                "seed {seed} block {block} must run both endpoints, got {rungs:?}"
            );
        }

        // Every (block, rung) group is a complete set of four targets, so a balanced order cannot
        // hide a missing arm or control.
        for rung in ScreenRung::ALL {
            let arms = frozen
                .attempts()
                .iter()
                .filter(|attempt| attempt.rung() == rung && attempt.role() == RunRole::Arm)
                .count();
            let controls = frozen
                .attempts()
                .iter()
                .filter(|attempt| attempt.rung() == rung && attempt.role() == RunRole::Control)
                .count();
            assert_eq!(
                (arms, controls),
                (4, 4),
                "seed {seed} rung {rung:?} must run four arms and four controls across both blocks"
            );
        }
    }
}
