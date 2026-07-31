//! The frozen inventory is exactly the two predeclared replicates, and they are distinct slots.

use crate::indexed_sender_view_calibration_pilot::attempt_inventory::{
    AttemptInventory, CALIBRATION_ATTEMPT_COUNT,
};

/// Coverage: the count the spec's decision names, and the distinctness the decision rule depends on.
///
/// SSOT §567 authorizes "exactly two" replicates and §569 evaluates every candidate `W` against
/// *both* series; §568 forbids a hidden retry. An inventory that emitted one replicate twice would
/// still produce two attempts and a two-line ledger, and the between-attempt disagreement the
/// decision rule weighs would be measuring one replicate against itself — silently, and with a ledger
/// that looks complete. So the count alone is not the property worth pinning; the pairing of count
/// and distinctness is.
#[test]
fn frozen_predeclares_two_distinct_slots() {
    let inventory = AttemptInventory::frozen().expect("the frozen inventory seals");
    let attempts = inventory.attempts();

    assert_eq!(
        attempts.len(),
        CALIBRATION_ATTEMPT_COUNT,
        "the spec authorizes exactly {CALIBRATION_ATTEMPT_COUNT} calibration attempts"
    );
    assert_eq!(
        CALIBRATION_ATTEMPT_COUNT, 2,
        "two, because one series cannot exhibit the between-attempt disagreement the decision rule \
         weighs"
    );

    for (position, attempt) in attempts.iter().enumerate() {
        for (other_position, other) in attempts.iter().enumerate().skip(position + 1) {
            assert!(
                !attempt.same_logical_slot(*other),
                "attempts {position} and {other_position} address the same logical slot, so the two \
                 series would be one replicate recorded twice"
            );
        }
    }
}
