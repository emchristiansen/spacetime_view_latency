//! An ordinary window, and one at the top of the admissible range, both convert exactly.

use crate::view_read_set_campaign::subscriber_delivery::wire_counter_snapshot::WireCounterSnapshot;

/// Coverage: the reduction is two subtractions, and both are meant to be exact rather than
/// approximate. The second half is the boundary that matters: `2^53` is admissible, and a window
/// reaching exactly it must convert rather than be refused, since the refusal is for sums that may
/// *already* have rounded — not for the largest one that provably has not.
#[test]
fn a_clean_window_converts_both_counters_exactly() {
    let open = WireCounterSnapshot {
        frames: 12,
        byte_sum: 3_500.0,
    };
    let close = WireCounterSnapshot {
        frames: 41,
        byte_sum: 91_204.0,
    };

    let delivery = close
        .since(&open)
        .expect("two whole-numbered sums well inside the exact-integer range reduce");
    assert_eq!(
        delivery.frames(),
        29,
        "the window's frames are the difference of the two cumulative counts"
    );
    assert_eq!(
        delivery.bytes(),
        87_704,
        "the window's bytes are the difference of the two cumulative sums"
    );

    let open = WireCounterSnapshot {
        frames: 0,
        byte_sum: 0.0,
    };
    let close = WireCounterSnapshot {
        frames: 1,
        byte_sum: 9_007_199_254_740_992.0,
    };

    let delivery = close
        .since(&open)
        .expect("a sum of exactly 2^53 is the largest one that is still provably unrounded");
    assert_eq!(
        delivery.bytes(),
        9_007_199_254_740_992,
        "the boundary sum converts to its own exact integer, losing nothing"
    );
}
