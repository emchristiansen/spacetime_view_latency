//! One deterministic seeding write, as a typed intent independent of the client.

use spacetimedb_sdk::Identity;

/// A single logical seeding write to be applied through the module's insertion reducers.
///
/// The dataset ([`crate::dataset::seed_plan::SeedPlan`]) produces an ordered list of these
/// so the seeding data is decided independently of the live client that issues it. A
/// Chronicle pair is one logical row — one `chronicle_message` plus its matching
/// `message_visibility` — so the `(viewer, message_uuid)` pair-uniqueness invariant is
/// expressed here as a single unit rather than two loosely-related inserts.
#[derive(Debug, Clone, Copy)]
pub(crate) enum SeedOp {
    /// Insert one `message` row with the fixed payload.
    Message { id: u64, sender: Identity },
    /// Insert one logical Chronicle pair: a `chronicle_message` with `uuid = key` and its
    /// matching `message_visibility` with `(viewer, message_uuid = key)`.
    ChroniclePair { key: u64, viewer: Identity },
}
