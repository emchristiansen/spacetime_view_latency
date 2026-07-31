//! Shared fixture: observed rows, and the identity they are owned by.

use spacetimedb_sdk::Identity;

use crate::module_artifact::bindings::EntityOwner;

/// The stand-in owner identity these tests' rows carry.
///
/// Derived through the same `from_claims` path the harness's own fixture identities use, so the
/// canonical hex it encodes to is a real identity's spelling rather than a hand-written string.
pub(super) fn owner() -> Identity {
    Identity::from_claims(
        "view-read-set-experiment-fixture",
        "observed-row-set-fixture-owner",
    )
}

/// One observed row at `entity_uuid` carrying `record`, owned by [`owner`].
pub(super) fn row(entity_uuid: u64, record: &str) -> EntityOwner {
    EntityOwner {
        entity_uuid,
        owner: owner(),
        record: record.to_string(),
    }
}
