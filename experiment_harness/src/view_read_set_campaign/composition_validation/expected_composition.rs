//! Exactly what one role must observe at one scale point, in one phase of the measured schedule.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its guarantee is
//! exactly that its sole constructor derived it. A private field is visible to its declaring module
//! **and every descendant**, so a `#[cfg(test)] mod tests` child — or any child added later — could
//! write the struct literal and *fit an expectation to an observation*, which is the one thing
//! validator input must never be. `sealed` has no children, so
//! [`ExpectedComposition::required`] really is the only door.

mod sealed {
    use serde::Serialize;
    use spacetimedb_sdk::Identity;

    use crate::plan::run_role::RunRole;
    use crate::view_read_set_campaign::campaign_params::{
        GLOBAL_KEY_BASE, OWNED_KEY_BASE, SEEDED_ROW_PAYLOAD, SUBSCRIBER_VISIBLE_ROWS_BASELINE,
    };
    use crate::view_read_set_campaign::composition_validation::expected_foreign_visibility::ExpectedForeignVisibility;
    use crate::view_read_set_campaign::composition_validation::expected_payload_state::ExpectedPayloadState;
    use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
    use crate::view_read_set_campaign::scale_point::ScalePoint;

    /// The exact result set a role is required to observe at a scale point, at a stated phase.
    ///
    /// **This is validator input, not a finding.** Every field is private to this childless module
    /// and [`Self::required`] is its only constructor, so an expectation is always derived from an
    /// attempt's own scale point, role, seeded identities, and phase rather than assembled to fit an
    /// observation. That derivation is all it guarantees: it carries no guarantee whatsoever about
    /// what any server actually returned, and only
    /// [`ValidatedComposition`](crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition)
    /// does. It is recorded in full so that a reader can re-derive the check rather than trust it.
    ///
    /// **Composition, not cardinality.** The completed Pilot established that a count-only check
    /// passes a Control returning the right *number* of wrong rows, so an expectation names key
    /// ranges, the canonical-hex owner each range must carry, and the payload state. Because the
    /// entity key is the table's primary key, distinct rows have distinct keys — so range membership
    /// plus a per-range count pins the exact expected key set without materializing it.
    ///
    /// **Phase, not one fixed payload.** The measured mutation changes payloads by design, so a
    /// single `payload` field could only ever be right before the first measured write. Naming the
    /// phase through [`ExpectedPayloadState`] is what lets the before-E2 seeded state and the
    /// after-batch derived state both be expressed without contradiction: the seeded payload remains
    /// recorded here because the foreign slice carries it in *every* phase, and the owned slice's
    /// phase-dependent payloads are derived from the schedule the state names.
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
        seeded_payload: String,
        payload_state: ExpectedPayloadState,
    }

    impl ExpectedComposition {
        /// What `role` must observe at `scale` once the schedule has reached `payload_state`,
        /// derived from the two seeded identities and the frozen key layout rather than supplied
        /// alongside them.
        ///
        /// Deriving it from the attempt's own identity is what stops an expectation from drifting
        /// away from the attempt it describes. The foreign visibility follows from the role by a
        /// total match over [`RunRole`]'s two variants — the Arm's sender-scoped view must show none
        /// of the foreign slice, the Control's base-table subscription all of it — so neither case
        /// can be omitted.
        ///
        /// The phase is a parameter rather than derived because it is a fact about *when* the
        /// observation was taken, which only the driver knows: the same attempt, role, and scale
        /// point yield a seeded expectation before E2 and a derived one after E1.
        ///
        /// **Which slice the swept quantity sizes** comes out of a total match on the scale point's
        /// own axis, not from reading [`ScalePoint::scale`] into one fixed field. The frozen
        /// quantity means different things per axis — on [`ExperimentAxis::UnrelatedGlobalRows`] it
        /// is the *foreign* row count while the owned slice stays at its baseline, and on a
        /// subscriber-visible axis it would be the other way round. Matching here means a second
        /// axis fails to compile at this function rather than silently sizing the wrong slice and
        /// producing an expectation that passes against the wrong observation.
        ///
        /// **Both range ends are checked additions**, even though
        /// [`campaign_params`](crate::view_read_set_campaign::campaign_params) already proves at
        /// compile time that the owned slice ends at or below `GLOBAL_KEY_BASE` and that the
        /// foreign slice cannot overflow at the top of the frozen ladder. That proof is written far
        /// from here and holds over the frozen constants alone; this derivation additionally reads
        /// [`ScalePoint::scale`], so a later axis or ladder breaking either premise would wrap a
        /// range end into a low key and yield an expectation that every correct observation fails
        /// against, for an unrelated-looking reason. Failing loud at the derivation keeps the
        /// diagnosis where the assumption drifted.
        pub(crate) fn required(
            scale: ScalePoint,
            role: RunRole,
            owned_owner: Identity,
            foreign_owner: Identity,
            payload_state: ExpectedPayloadState,
        ) -> Self {
            let (owned_rows, foreign_rows_seeded) = match scale.axis() {
                ExperimentAxis::UnrelatedGlobalRows => {
                    (SUBSCRIBER_VISIBLE_ROWS_BASELINE, scale.scale())
                }
            };
            let foreign_visibility = match role {
                RunRole::Arm => ExpectedForeignVisibility::None,
                RunRole::Control => ExpectedForeignVisibility::All {
                    rows: foreign_rows_seeded,
                },
            };
            Self {
                scale,
                owned_key_start: OWNED_KEY_BASE,
                owned_key_end: OWNED_KEY_BASE
                    .checked_add(owned_rows)
                    .expect("the owned slice ends below the frozen global key base"),
                owned_owner_hex: owned_owner.to_hex().to_string(),
                owned_rows,
                foreign_key_start: GLOBAL_KEY_BASE,
                foreign_key_end: GLOBAL_KEY_BASE
                    .checked_add(foreign_rows_seeded)
                    .expect("the foreign slice ends below the top of the key space"),
                foreign_owner_hex: foreign_owner.to_hex().to_string(),
                foreign_rows_seeded,
                foreign_visibility,
                seeded_payload: SEEDED_ROW_PAYLOAD.to_string(),
                payload_state,
            }
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

        /// How many of the measured identity's own rows this role must observe. Constant across
        /// every phase: the measured mutation updates rows in place and never changes cardinality.
        pub(crate) fn owned_rows(&self) -> u64 {
            self.owned_rows
        }

        /// The half-open key range `[start, end)` the other identity's rows occupy — disjoint from
        /// [`Self::owned_key_range`], so a primary-key collision between the two slices is
        /// impossible.
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

        /// The payload every row is seeded with. Foreign rows carry it in every phase, because no
        /// measured write ever targets the foreign slice; owned rows carry it only before the first
        /// measured write.
        pub(crate) fn seeded_payload(&self) -> &str {
            &self.seeded_payload
        }

        /// Which phase of the measured schedule this expectation describes, and therefore what the
        /// owned slice's payloads must be.
        pub(crate) fn payload_state(&self) -> ExpectedPayloadState {
            self.payload_state
        }
    }
}

pub(crate) use sealed::ExpectedComposition;
