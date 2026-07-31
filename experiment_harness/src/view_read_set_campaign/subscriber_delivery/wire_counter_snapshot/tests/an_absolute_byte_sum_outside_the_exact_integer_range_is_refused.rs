//! Every unusable absolute sum is refused, at whichever end of the window it appears.

use crate::view_read_set_campaign::subscriber_delivery::wire_counter_snapshot::WireCounterSnapshot;

/// The six ways an accumulated sum can be unusable: not a number, either infinity, negative,
/// fractional, and past the point where consecutive integers stop being representable.
const UNUSABLE: [f64; 6] = [
    f64::NAN,
    f64::INFINITY,
    f64::NEG_INFINITY,
    -1.0,
    2.5,
    9_007_199_254_740_994.0,
];

/// Coverage: this is the check that a delta-only gate would miss. Once an accumulating `f64` has
/// left the consecutive-integer range, the *difference* of two such readings is small, whole, and
/// entirely plausible — and quietly wrong, because the accumulation already rounded. So both ends
/// are gated before anything is subtracted, and this proves each end is gated on its own.
///
/// One entity for all six values at both ends because it is one claim about one gate: an absolute
/// reading is admissible only if it is a finite, non-negative, whole number no greater than `2^53`.
/// Splitting it per clause would assert the same mechanism six times over.
#[test]
fn an_absolute_byte_sum_outside_the_exact_integer_range_is_refused() {
    for unusable in UNUSABLE {
        let open = WireCounterSnapshot {
            frames: 1,
            byte_sum: unusable,
        };
        let close = WireCounterSnapshot {
            frames: 2,
            byte_sum: 4_096.0,
        };
        let error = close
            .since(&open)
            .expect_err("an unusable sum at the window's open must be refused");
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains("at the meter's open"),
            "the refusal must name the reading it rejected, got {rendered}"
        );

        let open = WireCounterSnapshot {
            frames: 1,
            byte_sum: 4_096.0,
        };
        let close = WireCounterSnapshot {
            frames: 2,
            byte_sum: unusable,
        };
        let error = close
            .since(&open)
            .expect_err("an unusable sum at the window's close must be refused");
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains("at the meter's close"),
            "the refusal must name the reading it rejected, got {rendered}"
        );
    }
}
