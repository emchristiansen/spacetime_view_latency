//! One `indexed_control_activity` row a paced append batch will write.

use spacetimedb_sdk::{Identity, Timestamp};

/// The complete argument list of one paced single-row append, prepared before the batch begins.
///
/// **Prebuilt on purpose.** The paced channel's estimand is issue-to-visible, so every value a write
/// needs is constructed before its clock starts. Handing the whole batch's rows in as a slice takes
/// that further than the campaign's per-iteration preparation does: the caller's frozen row recipe —
/// key arithmetic, timestamp derivation, owner and control attribution — runs once, entirely outside
/// the measured window, and the sampling loop does nothing per sample but copy four `Copy` fields.
///
/// It also keeps the recipe where it belongs. This type carries no default and no derivation of its
/// own, so a caller cannot leave a column to the client to invent; the population an append belongs
/// to is decided by whoever froze it, and the client only writes what it is given.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PacedAppend {
    /// The row's primary key, and the key its visibility is matched on.
    pub(crate) id: u64,
    /// The row's timestamp, supplied rather than clock-derived so a seeded state is reproducible.
    pub(crate) ts: Timestamp,
    /// The control this row is attributed to.
    pub(crate) control_uuid: u64,
    /// The identity that will own this row, and so whether a sender-scoped view returns it.
    pub(crate) user_identity: Identity,
}
