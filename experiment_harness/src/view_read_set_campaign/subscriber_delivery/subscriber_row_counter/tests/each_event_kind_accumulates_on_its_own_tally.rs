//! Each delivery kind lands on its own tally and nowhere else.

use crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counter::SubscriberRowCounter;

/// Coverage: the three counts are the same type and are recorded by three near-identical one-line
/// methods, so the mistake this catches is a body incrementing its neighbour's tally. Three distinct
/// counts make that visible; equal counts would pass either way.
///
/// The distinction is load-bearing rather than decorative: inserts are the cold subscription's
/// snapshot, updates are what the measured mutation produces, and a delete would mean the fixed
/// cardinality the protocol holds constant did not hold. A crossed wire here would misreport which
/// of those happened.
#[test]
fn each_event_kind_accumulates_on_its_own_tally() {
    let counter = SubscriberRowCounter::new();

    for _ in 0..3 {
        counter.record_insert();
    }
    for _ in 0..5 {
        counter.record_update();
    }
    counter.record_delete();

    let counts = counter.snapshot();
    assert_eq!(
        counts.inserts(),
        3,
        "three deliveries were recorded as inserts"
    );
    assert_eq!(
        counts.updates(),
        5,
        "five deliveries were recorded as updates"
    );
    assert_eq!(counts.deletes(), 1, "one delivery was recorded as a delete");
}
