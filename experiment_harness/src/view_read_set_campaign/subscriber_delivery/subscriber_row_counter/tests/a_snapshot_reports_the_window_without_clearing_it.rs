//! Reading the window twice reports the window twice, not a window and then nothing.

use crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counter::SubscriberRowCounter;

/// Coverage: this is the one deliberate deviation from the Pilot's per-dose counter, which drains.
/// One attempt owns one window and reads it once, so a draining read would make any second
/// observation report zeroes — counts indistinguishable from a connection that delivered nothing,
/// and fabricated rather than merely stale. Proving the second read equals the first is what pins
/// `snapshot` to a load.
///
/// The third read after further deliveries proves the tallies kept accumulating rather than being
/// frozen by the earlier reads, which is the other way a non-draining read could be wrong.
#[test]
fn a_snapshot_reports_the_window_without_clearing_it() {
    let counter = SubscriberRowCounter::new();
    counter.record_insert();
    counter.record_insert();
    counter.record_update();

    let first = counter.snapshot();
    let second = counter.snapshot();
    assert_eq!(
        first, second,
        "reading the window must not consume it; the second read reported {second:?} after \
         {first:?}"
    );

    counter.record_delete();
    let third = counter.snapshot();
    assert_eq!(
        (third.inserts(), third.updates(), third.deletes()),
        (2, 1, 1),
        "the tallies keep accumulating across reads rather than restarting from either one"
    );
}
