//! The paired before/after expectation one attempt's final composition check is judged against.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because minting the pair *as a
//! whole* is the only thing that keeps its two sides phase-, role-, and scale-matched. A private
//! field is visible to its declaring module **and every descendant**, so a `#[cfg(test)] mod tests`
//! child — or any child added later — could write the struct literal and assemble exactly the
//! mismatched pair this type exists to prevent. `sealed` has no children, so
//! [`CompositionTransitionExpectation::required_final`] really is the only door.

mod sealed {
    use serde::Serialize;
    use spacetimedb_sdk::Identity;

    use crate::plan::run_role::RunRole;
    use crate::view_read_set_campaign::composition_validation::expected_composition::ExpectedComposition;
    use crate::view_read_set_campaign::composition_validation::expected_payload_state::ExpectedPayloadState;
    use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
    use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;
    use crate::view_read_set_campaign::scale_point::ScalePoint;

    /// The two phase-matched expectations spanning an attempt's whole measured schedule: seeded
    /// before any measured write, and the saturated channel's derived final state after the last
    /// one.
    ///
    /// **Why the pair is a type.** A single [`ExpectedComposition`] names one phase, so handing one
    /// to a validator alongside two differently-phased observations would apply that phase's rule to
    /// both — judging the after-set as if nothing had been written, or the before-set as if
    /// everything had. Two loose expectations would fix that but permit a different error: pairing
    /// the after-**E2** expectation with the final after-**E1** row set. Both batches walk the same
    /// ten owned keys with the same write indices and differ only in their channel tag, so that
    /// mismatch produces payloads of exactly the right shape and the wrong tag — the kind of near-miss
    /// a hand-assembled pair invites.
    ///
    /// So the transition is minted as a whole. [`Self::required_final`] derives both sides from one
    /// attempt's identity, role, and scale point, fixing `before` to
    /// [`Seeded`](crate::view_read_set_campaign::composition_validation::expected_payload_state::ExpectedPayloadState::Seeded)
    /// and `after` to the **saturated** channel's schedule — which is the final state because E1 runs
    /// last in the frozen execution order.
    ///
    /// **Who can construct it.** Both fields are private to this childless module and
    /// [`Self::required_final`] is the only constructor, so no other module can assemble a mismatched
    /// pair.
    ///
    /// **What it does not cover.** E2's per-sample visibility. The paced channel stops each sample
    /// when its change is observable in the subscriber cache, and that evidence lives in the paced
    /// channel's own samples — not here. This transition is the *composition* claim spanning the
    /// attempt, not a record of every intermediate state.
    ///
    /// **This is validator input, not a finding.** It asserts nothing about what any server returned;
    /// only
    /// [`ValidatedComposition`](crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition)
    /// does.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub(crate) struct CompositionTransitionExpectation {
        before: ExpectedComposition,
        after: ExpectedComposition,
    }

    impl CompositionTransitionExpectation {
        /// Derive both phases of one attempt's composition expectation from its own identity.
        ///
        /// Named `required_final` rather than `required` because the *final* transition is the only
        /// one it mints: before any measured write, and after the last one. An intermediate
        /// transition would be a different constructor with a different name, so neither can be
        /// mistaken for the other at a call site.
        ///
        /// Both sides come from one call to [`ExpectedComposition::required`] each, over the same
        /// scale, role, and identities, so the only thing that differs between them is the phase —
        /// which is the whole point of minting the pair together.
        pub(crate) fn required_final(
            scale: ScalePoint,
            role: RunRole,
            owned_owner: Identity,
            foreign_owner: Identity,
        ) -> Self {
            let saturated = MutationSchedule::of(MeasurementChannel::SaturatedQueueGrowthPerWrite)
                .expect("the saturated channel issues measured writes, so it has a schedule");
            Self {
                before: ExpectedComposition::required(
                    scale,
                    role,
                    owned_owner,
                    foreign_owner,
                    ExpectedPayloadState::Seeded,
                ),
                after: ExpectedComposition::required(
                    scale,
                    role,
                    owned_owner,
                    foreign_owner,
                    ExpectedPayloadState::AfterMeasuredBatch(saturated),
                ),
            }
        }

        /// The measured-write schedule the after phase is bound to.
        ///
        /// Read back out of the after-expectation rather than stored beside it, so there is no
        /// second copy of the schedule to disagree with the phase it describes. This is what lets
        /// the composition check derive the final mutation — its offset, its payload, and the
        /// payload it replaced — from the transition it was handed, instead of from a caller's
        /// separate claim about which write to look at.
        ///
        /// [`Self::required_final`] fixes the after phase to
        /// [`AfterMeasuredBatch`](ExpectedPayloadState::AfterMeasuredBatch) and is the only
        /// constructor, so the seeded arm is unreachable and says so loudly rather than inventing a
        /// schedule.
        pub(crate) fn after_schedule(&self) -> MutationSchedule {
            let ExpectedPayloadState::AfterMeasuredBatch(schedule) = self.after.payload_state()
            else {
                panic!(
                    "required_final fixes the after phase to the saturated batch, so a transition \
                     always names the schedule its final state was derived from"
                )
            };
            schedule
        }

        /// What must be observed before any measured write: the seeded payload on every row.
        pub(crate) fn before(&self) -> &ExpectedComposition {
            &self.before
        }

        /// What must be observed after the saturated batch confirms: the schedule's derived final
        /// payload on each owned key, the seeded payload still on every foreign row.
        pub(crate) fn after(&self) -> &ExpectedComposition {
            &self.after
        }

        /// The scale point both phases describe. `required_final` derives them from one scale point,
        /// so this is unambiguous.
        pub(crate) fn scale(&self) -> ScalePoint {
            self.before.scale()
        }
    }
}

pub(crate) use sealed::CompositionTransitionExpectation;
