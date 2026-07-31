//! Two admissible sums whose difference is negative are still refused.

use crate::view_read_set_campaign::subscriber_delivery::wire_counter_snapshot::WireCounterSnapshot;

/// Coverage: this is the one clause of the delta gate that can still fire. Once both absolute
/// readings are exact integers at most `2^53`, their difference is exact too — so the finite,
/// integral, and range clauses have nothing left to catch, and what remains is the sum having fallen
/// across the window. Retaining the whole gate rather than reducing it to a sign check is what keeps
/// the three impossible clauses impossible *because* of the earlier ones rather than by assumption.
///
/// The refusal must also carry both absolute readings, since "the sum went down" without them says
/// nothing about which end moved.
#[test]
fn a_byte_sum_that_fell_over_the_window_is_refused() {
    let open = WireCounterSnapshot {
        frames: 10,
        byte_sum: 5_000.0,
    };
    let close = WireCounterSnapshot {
        frames: 11,
        byte_sum: 4_000.0,
    };

    let error = close
        .since(&open)
        .expect_err("a cumulative byte sum cannot fall across one window");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("over the measured window"),
        "the refusal must name the delta as the unusable reading, got {rendered}"
    );
    assert!(
        rendered.contains("5000") && rendered.contains("4000"),
        "the refusal must carry both absolute sums, got {rendered}"
    );
}
