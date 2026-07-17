//! SDK logical delivery-event evidence for one dose.

use anyhow::{ensure, Context, Result};
use serde::Serialize;

/// The SDK's client-visible delivery events for one dose, excluding the initial subscription
/// snapshot: the signed net delivered row delta and the separate insert/delete/update event counts.
///
/// These are **logical/client-visible** events — the net change to the subscribed result set as the
/// SDK delivered it. They are never physical/materializer write counts and must not be read as a
/// proxy for hidden refresh work (spec: "Secondary signals"). The checked constructor enforces the
/// identity that inserts − deletes equals the supplied `queried_net_delta` (an update replaces a row
/// in place and does not change the row count), rejecting a disagreement. This proves agreement
/// *between the constructor's inputs* only; that `queried_net_delta` was itself truthfully computed
/// from the real post-write result set is the future measurement checker's provenance, not this
/// type's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct EventEvidence {
    delivered_net_row_delta: i64,
    inserts: u64,
    deletes: u64,
    updates: u64,
}

impl EventEvidence {
    /// Build the evidence, rejecting an event stream whose net delta disagrees with the queried
    /// post-write result-set net delta.
    pub(crate) fn checked(
        inserts: u64,
        deletes: u64,
        updates: u64,
        queried_net_delta: i64,
    ) -> Result<Self> {
        // `i128` cannot overflow for any `u64` inputs, so the identity check itself never wraps.
        let event_net = i128::from(inserts) - i128::from(deletes);
        ensure!(
            event_net == i128::from(queried_net_delta),
            "delivered event net row delta (inserts {inserts} − deletes {deletes} = {event_net}) \
             disagrees with the queried post-write net delta {queried_net_delta}"
        );
        let delivered_net_row_delta = i64::try_from(event_net)
            .context("delivered net row delta does not fit i64")?;
        Ok(Self {
            delivered_net_row_delta,
            inserts,
            deletes,
            updates,
        })
    }

    /// The signed net change to the subscribed result set delivered this dose.
    pub(crate) fn delivered_net_row_delta(self) -> i64 {
        self.delivered_net_row_delta
    }

    /// The number of insert events delivered this dose.
    pub(crate) fn inserts(self) -> u64 {
        self.inserts
    }

    /// The number of delete events delivered this dose.
    pub(crate) fn deletes(self) -> u64 {
        self.deletes
    }

    /// The number of update events delivered this dose.
    pub(crate) fn updates(self) -> u64 {
        self.updates
    }
}

#[cfg(test)]
mod tests;
