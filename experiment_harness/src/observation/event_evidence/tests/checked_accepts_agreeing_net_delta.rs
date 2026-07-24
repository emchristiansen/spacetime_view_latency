//! `checked` accepts an event stream whose inserts − deletes equals the queried net row delta,
//! independent of the update count (updates are in-place and do not change the row count).

use crate::observation::event_evidence::EventEvidence;

#[test]
fn checked_accepts_agreeing_net_delta() {
    let evidence = EventEvidence::checked(1_000, 250, 40, 750)
        .expect("inserts − deletes = 750 agrees with the queried net delta");
    assert_eq!(evidence.delivered_net_row_delta(), 750);
    assert_eq!(evidence.inserts(), 1_000);
    assert_eq!(evidence.deletes(), 250);
    assert_eq!(evidence.updates(), 40);
}
