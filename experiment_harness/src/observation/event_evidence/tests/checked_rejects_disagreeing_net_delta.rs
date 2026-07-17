//! `checked` rejects inputs whose inserts − deletes disagrees with the supplied `queried_net_delta`.
//! The type proves agreement *between its inputs*; whether `queried_net_delta` was truthfully
//! computed from the real result set is the future private checker's provenance, not this type's.

use crate::observation::event_evidence::EventEvidence;

#[test]
fn checked_rejects_disagreeing_net_delta() {
    // inserts − deletes = 1_000, but the queried result set reports a net delta of 999.
    EventEvidence::checked(1_000, 0, 0, 999)
        .expect_err("a net-delta disagreement must be rejected");
}
