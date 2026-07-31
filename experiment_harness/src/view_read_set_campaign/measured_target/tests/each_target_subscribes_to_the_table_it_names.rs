//! Each target's subscription query names its own table and not the other role's.

use crate::client::connected_client::{TABLE_ENTITY_OWNER, TABLE_ENTITY_OWNER_SENDER_VIEW};
use crate::view_read_set_campaign::measured_target::MeasuredTarget;

/// Coverage: `entity_owner` is a substring of `entity_owner_sender_view`, so a containment check
/// would accept a query built from the wrong constant. Full equality instead — a Control subscribing
/// to the view would return ten owned rows against a census expecting the whole swept slice.
///
/// The expected text is rebuilt from the shared constants rather than written as a literal: they are
/// the contract with the module's declared accessors, and a test asserting its own copy would keep
/// passing while that contract broke.
#[test]
fn each_target_subscribes_to_the_table_it_names() {
    assert_eq!(
        MeasuredTarget::EntityOwner.subscription_sql(),
        format!("SELECT * FROM {TABLE_ENTITY_OWNER}"),
        "the Control subscribes to the direct public base table, by its exact declared name"
    );
    assert_eq!(
        MeasuredTarget::EntityOwnerSenderView.subscription_sql(),
        format!("SELECT * FROM {TABLE_ENTITY_OWNER_SENDER_VIEW}"),
        "the Arm subscribes to the sender-scoped view, by its exact declared name"
    );
    assert_ne!(
        MeasuredTarget::EntityOwner.subscription_sql(),
        MeasuredTarget::EntityOwnerSenderView.subscription_sql(),
        "the two roles' subscriptions name different tables, which is the whole of their difference"
    );
}
