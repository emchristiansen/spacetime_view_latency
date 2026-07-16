//! One measurement role's deterministic slice of a run's seeded dataset.

use spacetimedb_sdk::Identity;

use crate::module_artifact::bindings::{ChronicleMessage, Message};
use crate::params::ROW_PAYLOAD;

/// One role's deterministic contribution to the dataset: exactly `count` rows keyed
/// contiguously from `key_base`, every row attributed to `identity`. Because the measured
/// and growth key bases are a full billion keys apart, two roles' primary keys can never
/// collide, and each role's `(identity, key)` rows are unique by construction.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RoleSlice {
    identity: Identity,
    key_base: u64,
    count: u64,
}

impl RoleSlice {
    pub(crate) fn new(identity: Identity, key_base: u64, count: u64) -> Self {
        Self {
            identity,
            key_base,
            count,
        }
    }

    pub(crate) fn identity(&self) -> Identity {
        self.identity
    }

    pub(crate) fn count(&self) -> u64 {
        self.count
    }

    /// The primary keys assigned to this slice's rows: `key_base .. key_base + count`.
    pub(crate) fn keys(&self) -> std::ops::Range<u64> {
        self.key_base..self.key_base + self.count
    }

    /// This slice's expected `message`-family rows (`id` = key, `sender` = identity, the
    /// fixed payload).
    pub(crate) fn expected_messages(&self) -> Vec<Message> {
        self.keys()
            .map(|id| Message {
                id,
                sender: self.identity,
                payload: ROW_PAYLOAD.to_string(),
            })
            .collect()
    }

    /// This slice's expected `chronicle_message`-family rows (`uuid` = key, the fixed
    /// payload). Chronicle rows carry no identity column; the viewer attribution lives in
    /// the paired `message_visibility` row.
    pub(crate) fn expected_chronicle(&self) -> Vec<ChronicleMessage> {
        self.keys()
            .map(|uuid| ChronicleMessage {
                uuid,
                payload: ROW_PAYLOAD.to_string(),
            })
            .collect()
    }
}
