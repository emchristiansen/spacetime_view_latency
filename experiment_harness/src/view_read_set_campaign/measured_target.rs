//! Which table one attempt's role subscribes to, reads back, and counts deliveries on.

use spacetimedb_sdk::{Table, TableWithPrimaryKey};

use crate::client::connected_client::{TABLE_ENTITY_OWNER, TABLE_ENTITY_OWNER_SENDER_VIEW};
use crate::module_artifact::bindings::{
    DbConnection, EntityOwner, EntityOwnerSenderViewTableAccess, EntityOwnerTableAccess,
};
use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::subscriber_delivery::delivery_counters::DeliveryCounters;
use crate::view_read_set_campaign::visibility_observer::VisibilityObserver;

/// The one table an attempt measures: what it subscribes to, reads back, counts deliveries on, and
/// stops a paced sample on.
///
/// Four concerns on one enum because they must name the same table or the evidence is about nothing
/// — counting deliveries on an unsubscribed table reports zero, and stopping a paced sample on a
/// table the write does not reach never stops. Each projection is a total match, so they cannot
/// drift; the discipline [`SubscribedTable`](crate::dataset::subscribed_table) states for the
/// historical campaign.
///
/// Derived from the role by [`Self::of`] rather than carried beside it: this choice of table *is*
/// the Arm/Control distinction, since the two roles seed identically, write identically, and are
/// judged by the same composition transition.
///
/// Both targets declare `entity_uuid` as a primary key, so both generated handles implement
/// [`TableWithPrimaryKey`] and deliver the in-place updates the measured mutation produces. The
/// generic helpers below take that bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeasuredTarget {
    /// The Arm: the module's sender-scoped view, whose read set is the candidate's own answer.
    EntityOwnerSenderView,
    /// The matched direct public-table Control.
    EntityOwner,
}

impl MeasuredTarget {
    /// The table this role measures.
    pub(crate) fn of(role: RunRole) -> Self {
        match role {
            RunRole::Arm => Self::EntityOwnerSenderView,
            RunRole::Control => Self::EntityOwner,
        }
    }

    /// The subscription query naming this target, from the same table-name constants the client's
    /// own helpers use — one contract with the module's declared accessors, not a second copy.
    pub(crate) fn subscription_sql(self) -> String {
        match self {
            Self::EntityOwnerSenderView => {
                format!("SELECT * FROM {TABLE_ENTITY_OWNER_SENDER_VIEW}")
            }
            Self::EntityOwner => format!("SELECT * FROM {TABLE_ENTITY_OWNER}"),
        }
    }

    /// Read this target's currently-subscribed rows out of the live client cache, issuing no new
    /// subscription. Both variants yield [`EntityOwner`] rows — the view projects the base table's
    /// row type unchanged, which is why one composition vocabulary judges both roles.
    pub(crate) fn read_back(self, conn: &DbConnection) -> Vec<EntityOwner> {
        match self {
            Self::EntityOwnerSenderView => conn.db.entity_owner_sender_view().iter().collect(),
            Self::EntityOwner => conn.db.entity_owner().iter().collect(),
        }
    }

    /// Register this target's three delivery callbacks, taking the counter handles already cloned
    /// because the reconnect calls this inside a measured interval.
    ///
    /// Registered before the cold subscription, so that subscription's initial snapshot counts as
    /// the delivery it is.
    pub(crate) fn register_delivery_callbacks(
        self,
        conn: &DbConnection,
        counters: DeliveryCounters,
    ) {
        match self {
            Self::EntityOwnerSenderView => {
                register_delivery(&conn.db.entity_owner_sender_view(), counters);
            }
            Self::EntityOwner => register_delivery(&conn.db.entity_owner(), counters),
        }
    }

    /// Register the paced channel's update observer, reporting **only** the updated row's primary
    /// key.
    ///
    /// The key is a `u64`, so the observer allocates nothing and encodes no row: adding serialization
    /// to the path E2 measures would perturb the estimand this observer exists to stop.
    pub(crate) fn observe_visible_updates(
        self,
        conn: &DbConnection,
        on_visible: impl FnMut(u64) + Send + 'static,
    ) -> VisibilityObserver {
        match self {
            Self::EntityOwnerSenderView => VisibilityObserver::EntityOwnerSenderView(
                observe_updated_keys(&conn.db.entity_owner_sender_view(), on_visible),
            ),
            Self::EntityOwner => VisibilityObserver::EntityOwner(observe_updated_keys(
                &conn.db.entity_owner(),
                on_visible,
            )),
        }
    }
}

/// Register one table handle's three delivery callbacks, each moving its own prebuilt handle.
fn register_delivery<T: TableWithPrimaryKey>(table: &T, counters: DeliveryCounters) {
    let DeliveryCounters {
        inserts,
        updates,
        deletes,
    } = counters;
    table.on_insert(move |_ctx, _row| inserts.record_insert());
    table.on_update(move |_ctx, _old, _new| updates.record_update());
    table.on_delete(move |_ctx, _row| deletes.record_delete());
}

/// Register one table handle's update callback, reporting each updated row's `entity_uuid`.
fn observe_updated_keys<T>(
    table: &T,
    mut on_visible: impl FnMut(u64) + Send + 'static,
) -> T::UpdateCallbackId
where
    T: TableWithPrimaryKey + Table<Row = EntityOwner>,
{
    table.on_update(move |_ctx, _old, new| on_visible(new.entity_uuid))
}

#[cfg(test)]
mod tests;
