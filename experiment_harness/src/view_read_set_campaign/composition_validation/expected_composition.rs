//! Exactly what one role must observe at one scale point.

use serde::Serialize;
use spacetimedb_sdk::Identity;

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::expected_foreign_visibility::ExpectedForeignVisibility;
use crate::view_read_set_campaign::scale_point::ScalePoint;

/// The exact result set a role is required to observe at a scale point.
///
/// **This is a claim to be checked, not a finding.** It is freely constructible and carries no
/// guarantee whatsoever about what any server actually returned; only
/// [`ValidatedComposition`](super::validated_composition::ValidatedComposition) does that. It is
/// recorded in full so that a reader can re-derive the check rather than trust it.
///
/// Composition, not cardinality: the completed Pilot established that a count-only check passes a
/// Control returning the right *number* of wrong rows, so an expectation names key ranges, the
/// canonical-hex owner each range must carry, and the one fixed payload. Because the entity key is
/// the table's primary key, distinct rows have distinct keys — so range membership plus a per-range
/// count pins the exact expected key set without materializing it.
///
/// Every field is serialized. The key ranges are stored as explicit inclusive-start/exclusive-end
/// bounds rather than a `Range`, and the identities as canonical hex rather than as opaque SDK
/// values, precisely so the whole expectation survives into the ledger and can be audited there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ExpectedComposition {
    scale: ScalePoint,
    owned_key_start: u64,
    owned_key_end: u64,
    owned_owner_hex: String,
    owned_rows: u64,
    foreign_key_start: u64,
    foreign_key_end: u64,
    foreign_owner_hex: String,
    foreign_rows_seeded: u64,
    foreign_visibility: ExpectedForeignVisibility,
    payload: String,
}

impl ExpectedComposition {
    /// What `role` must observe at `scale`, derived from the two seeded identities and the frozen
    /// key layout rather than supplied alongside them.
    ///
    /// Deriving it from the attempt's own identity is what stops an expectation from drifting away
    /// from the attempt it describes. The foreign visibility follows from the role by a total match
    /// over [`RunRole`]'s two variants — the Arm's sender-scoped view must show none of the foreign
    /// slice, the Control's base-table subscription all of it — so neither case can be omitted.
    ///
    /// **Phase 1 boundary.** The key layout this reads lands with the driver in Phase 2.
    pub(crate) fn required(
        scale: ScalePoint,
        role: RunRole,
        owned_owner: Identity,
        foreign_owner: Identity,
    ) -> Self {
        let _ = (scale, role, owned_owner, foreign_owner);
        todo!(
            "Phase 2: derive the owned and foreign key ranges from this scale point's frozen \
             quantity and the campaign key bases, record both identities as canonical hex, and set \
             foreign_visibility by a total match on RunRole — None for the Arm, All for the Control"
        )
    }

    /// The scale point this expectation applies at.
    pub(crate) fn scale(&self) -> ScalePoint {
        self.scale
    }

    /// The half-open key range `[start, end)` the measured identity's own rows occupy.
    pub(crate) fn owned_key_range(&self) -> (u64, u64) {
        (self.owned_key_start, self.owned_key_end)
    }

    /// The canonical-hex identity every row in the owned range must be owned by.
    pub(crate) fn owned_owner_hex(&self) -> &str {
        &self.owned_owner_hex
    }

    /// How many of the measured identity's own rows this role must observe.
    pub(crate) fn owned_rows(&self) -> u64 {
        self.owned_rows
    }

    /// The half-open key range `[start, end)` the other identity's rows occupy — disjoint from
    /// [`Self::owned_key_range`], so a primary-key collision between the two slices is impossible.
    pub(crate) fn foreign_key_range(&self) -> (u64, u64) {
        (self.foreign_key_start, self.foreign_key_end)
    }

    /// The canonical-hex identity every row in the foreign range must be owned by.
    pub(crate) fn foreign_owner_hex(&self) -> &str {
        &self.foreign_owner_hex
    }

    /// How much of the foreign slice this role must observe.
    pub(crate) fn foreign_visibility(&self) -> ExpectedForeignVisibility {
        self.foreign_visibility
    }

    /// The one fixed payload every seeded row carries, so payload width never confounds a
    /// comparison. Distinct from the measured mutation's payloads, which vary by construction.
    pub(crate) fn payload(&self) -> &str {
        &self.payload
    }
}
