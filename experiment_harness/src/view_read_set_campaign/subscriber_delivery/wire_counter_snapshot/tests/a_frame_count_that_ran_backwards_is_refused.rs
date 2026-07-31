//! A close reading below the open one is a reset, not a window.

use crate::view_read_set_campaign::subscriber_delivery::wire_counter_snapshot::WireCounterSnapshot;

/// Coverage: the histogram's sample count is cumulative for the life of the process, so it only ever
/// increases. A smaller value at close therefore does not mean "negative traffic" — it means the two
/// readings do not bound one window, and no subtraction of them is meaningful. Wrapping or
/// saturating here would turn that into a plausible number.
#[test]
fn a_frame_count_that_ran_backwards_is_refused() {
    let open = WireCounterSnapshot {
        frames: 900,
        byte_sum: 10_000.0,
    };
    let close = WireCounterSnapshot {
        frames: 12,
        byte_sum: 20_000.0,
    };

    let error = close
        .since(&open)
        .expect_err("a cumulative count cannot decrease across one window");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("900") && rendered.contains("12"),
        "the refusal must name both readings, got {rendered}"
    );
    assert!(
        rendered.contains("reset"),
        "the refusal must say what a decrease actually indicates, got {rendered}"
    );
}
