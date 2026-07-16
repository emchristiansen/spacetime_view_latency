//! The published database identity.

use serde::{Serialize, Serializer};
use spacetimedb_sdk::Identity;

/// The SpacetimeDB identity of the published experiment database, recorded in the run
/// manifest. Wraps the SDK's own `Identity` type rather than a hex string, so the value
/// carries its domain meaning; serialized as canonical hex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DatabaseIdentity(Identity);

impl DatabaseIdentity {
    pub(crate) fn new(identity: Identity) -> Self {
        Self(identity)
    }

    /// The published database identity.
    pub(crate) fn identity(&self) -> &Identity {
        &self.0
    }
}

impl Serialize for DatabaseIdentity {
    /// Serialized as the canonical lowercase hex identity.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_hex().to_string())
    }
}
