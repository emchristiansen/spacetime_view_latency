//! The registered row-update observer the paced channel removes before the saturated channel runs.

use spacetimedb_sdk::TableWithPrimaryKey;

use crate::module_artifact::bindings::{
    DbConnection, EntityOwnerSenderViewTableAccess, EntityOwnerSenderViewUpdateCallbackId,
    EntityOwnerTableAccess, EntityOwnerUpdateCallbackId,
};

/// A registered `on_update` callback on one attempt's measured target, retained so it can be
/// cancelled.
///
/// The two generated id types are newtypes whose fields are private to their own generated modules,
/// so an id can only have come from `on_update` on the table its variant names: [`Self::remove`]
/// cannot be pointed at the other table.
///
/// Removed at the end of the paced channel, unlike the delivery counter callbacks beside it, because
/// it is instrumentation *of* that channel — the saturated channel must not pay a channel send per
/// delivered row for a signal nothing reads.
pub(crate) enum VisibilityObserver {
    EntityOwnerSenderView(EntityOwnerSenderViewUpdateCallbackId),
    EntityOwner(EntityOwnerUpdateCallbackId),
}

impl VisibilityObserver {
    /// Cancel this observer, consuming it so a second cancellation of the same id cannot exist.
    pub(crate) fn remove(self, conn: &DbConnection) {
        match self {
            Self::EntityOwnerSenderView(callback) => {
                remove_update_callback(&conn.db.entity_owner_sender_view(), callback);
            }
            Self::EntityOwner(callback) => {
                remove_update_callback(&conn.db.entity_owner(), callback);
            }
        }
    }
}

/// Cancel one table handle's update callback. Generic because each generated handle implements two
/// traits carrying an identically named `remove_on_update`, so the bound is what disambiguates —
/// as it does for [`SubscribedTable`](crate::dataset::subscribed_table)'s registration helpers.
fn remove_update_callback<T: TableWithPrimaryKey>(table: &T, callback: T::UpdateCallbackId) {
    table.remove_on_update(callback);
}
