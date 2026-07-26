//! The artifact of having actually checked one attempt's observed result sets.
//!
//! **Module topology is the enforcement mechanism here, not a comment.** The finding and its witness
//! live together in the private, *childless* inline module [`sealed`]. Rust makes a private field
//! visible to its declaring module **and every descendant**, so declaring
//! [`ValidatedComposition`] beside a `#[cfg(test)] mod tests` child — or any child added later —
//! would let that module write the struct literal and assert a composition finding that no comparison
//! ever produced. `sealed` has no children, so the comparison really is the only door.
//!
//! `MutationWitness` is sealed *with* it rather than beside it: it is the finding's own payload, it
//! must stay unassemblable independently of the check that produced it, and only the finding names
//! it, so it is not re-exported at all.

mod sealed {
    use anyhow::Result;
    use serde::Serialize;

    use crate::view_read_set_campaign::composition_validation::composition_transition_expectation::CompositionTransitionExpectation;
    use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
    use crate::view_read_set_campaign::mutation_schedule::OwnedSliceOffset;
    use crate::view_read_set_campaign::scale_point::ScalePoint;

    /// The record of a composition check that was performed, pointing at the rows it checked.
    ///
    /// **Who can construct it.** Every field is private to this childless module and
    /// [`Self::validate`] is the only constructor, declared alongside them here. No other module —
    /// sibling, parent, or elsewhere in the crate — can build one, by struct literal or otherwise, so
    /// a composition finding cannot be asserted without going through the comparison. Field privacy
    /// *within a leaf module* is what enforces this; a `pub(super)` constructor, or a private field
    /// in a module that has children, would have been reachable from code that never ran the check.
    ///
    /// **Why one transition rather than two expectations.** The measured schedule changes payloads by
    /// design, so the before and after observations must be judged against different expected states.
    /// Taking two independent expectations would fix that and permit a worse error — pairing the
    /// after-E2 expectation with the final after-E1 row set, or two expectations from different roles
    /// or scale points. [`CompositionTransitionExpectation`] is minted as a whole from one attempt's
    /// identity, so the two sides are phase-matched, role-matched, and scale-matched before this
    /// function ever sees them.
    ///
    /// **What [`Self::validate`] checks.** Each observation against its own side of the transition:
    /// every row falls in one of the two preregistered key ranges; each carries that range's expected
    /// owner; the owned count is exact and unchanged across phases, since the schedule updates in
    /// place; the foreign count matches
    /// [`ExpectedForeignVisibility`](crate::view_read_set_campaign::composition_validation::expected_foreign_visibility::ExpectedForeignVisibility),
    /// which for the Arm is zero and is the candidate's security gate; and payloads match the phase —
    /// seeded everywhere before, and the schedule's derived final payload on each owned key after,
    /// with the foreign slice seeded throughout. It then requires the witness row to appear in both
    /// observations with a changed payload, since the pinned SpacetimeDB source elides a
    /// byte-identical update outright.
    ///
    /// **What it does not cover.** E2's per-sample visibility, which lives in the paced channel's own
    /// samples. This is the composition claim spanning the attempt, not a record of every
    /// intermediate state.
    ///
    /// **What makes it auditable.** The two [`ObservedRowSet`]s are content-addressed artifacts on
    /// disk, not summaries, so a reader can fetch the exact rows and re-run every comparison rather
    /// than trusting that a validator once returned `Ok`. The embedded transition, the per-range
    /// counts, and the witness row's key and both payloads are recorded so the check can be
    /// reproduced without reading this module.
    ///
    /// **What it does not claim.** It does not establish that the observations came from a real
    /// server; that is a property of the driver's measurement path. It claims only that these
    /// specific retained rows satisfy this specific recorded transition.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct ValidatedComposition {
        expected: CompositionTransitionExpectation,
        before: ObservedRowSet,
        after: ObservedRowSet,
        observed_owned_rows: u64,
        observed_foreign_rows: u64,
        mutation: MutationWitness,
        delivered_rows: u64,
        client_cache_rows: u64,
        subscription_handles: u32,
    }

    /// The concrete before/after state of the owned row the check used as its visibility witness.
    ///
    /// Private to this childless module, so it cannot be assembled independently of the check that
    /// produced it. It records the key and both payloads rather than asserting that a change was
    /// seen, so "the mutation was visible" is something a reader verifies against two values instead
    /// of a flag they must believe — and the two values are also in the retained row sets, so the
    /// record can be cross-checked.
    #[derive(Debug, Clone, Serialize)]
    struct MutationWitness {
        entity_key: u64,
        payload_before: String,
        payload_after: String,
    }

    impl ValidatedComposition {
        /// Check both observed result sets against their own side of `expected`, minting the
        /// artifact only if every applicable gate holds.
        ///
        /// The witness is named by an [`OwnedSliceOffset`] rather than a raw key: the witness must be
        /// a row the schedule actually targets, and an arbitrary `u64` could name a foreign row —
        /// which no measured write ever touches, so its payload is unchanged by construction and it
        /// would witness visibility that never happened. The offset can only have come from a walk of
        /// the frozen owned slice, and the owned key is derived from it.
        ///
        /// **Phase 1 boundary.** The comparison lands in Phase 2. The signature is what fixes which
        /// observations a finding must derive from: it takes the retained row sets rather than
        /// counts, so no caller can pre-reduce the evidence into something this function cannot
        /// contradict.
        pub(crate) fn validate(
            expected: CompositionTransitionExpectation,
            before: ObservedRowSet,
            after: ObservedRowSet,
            witness: OwnedSliceOffset,
            delivered_rows: u64,
            client_cache_rows: u64,
            subscription_handles: u32,
        ) -> Result<Self> {
            let _ = (
                expected,
                before,
                after,
                witness,
                delivered_rows,
                client_cache_rows,
                subscription_handles,
            );
            todo!(
                "Phase 2: census each retained row set against its own side of the transition, \
                 rejecting any row outside both key ranges and any row whose owner is not its \
                 range's expected identity; require the owned count to be exact and identical across \
                 the two phases, since the schedule updates in place; require the foreign count to \
                 match ExpectedForeignVisibility (zero for the Arm — the security gate) and every \
                 foreign row to carry the seeded payload in both phases; match each side's \
                 ExpectedPayloadState, requiring the seeded payload on every owned row for Seeded \
                 and the schedule's final_payload_at_offset for each owned slice offset after the \
                 measured batch; and require the witness key, derived as OWNED_KEY_BASE plus the \
                 witness offset, to be present in both observations with a changed payload, since a \
                 byte-identical update is elided at the pinned commit"
            )
        }

        /// The scale point this finding was checked at, taken from the transition that fixed it.
        pub(crate) fn scale(&self) -> ScalePoint {
            self.expected.scale()
        }

        /// The phase-matched transition this finding was checked against, embedded so the artifact
        /// is self-contained.
        pub(crate) fn expected(&self) -> &CompositionTransitionExpectation {
            &self.expected
        }

        /// The content-addressed row sets this finding was derived from — where a reader goes to
        /// re-run the comparison.
        pub(crate) fn observations(&self) -> (&ObservedRowSet, &ObservedRowSet) {
            (&self.before, &self.after)
        }

        /// How many of the measured identity's own rows were observed.
        pub(crate) fn observed_owned_rows(&self) -> u64 {
            self.observed_owned_rows
        }

        /// How many foreign rows were observed. Zero is the Arm's passing security gate.
        pub(crate) fn observed_foreign_rows(&self) -> u64 {
            self.observed_foreign_rows
        }

        /// The witness row's key and its payload before and after the measured schedule ran — the
        /// concrete values behind the visibility claim.
        pub(crate) fn mutation(&self) -> (u64, &str, &str) {
            (
                self.mutation.entity_key,
                &self.mutation.payload_before,
                &self.mutation.payload_after,
            )
        }

        /// Rows delivered to this subscriber, recorded as supporting evidence rather than a trend
        /// estimand.
        pub(crate) fn delivered_rows(&self) -> u64 {
            self.delivered_rows
        }

        /// Rows resident in the client cache, recorded as supporting evidence.
        pub(crate) fn client_cache_rows(&self) -> u64 {
            self.client_cache_rows
        }

        /// Live subscription handles, recorded as supporting evidence.
        pub(crate) fn subscription_handles(&self) -> u32 {
            self.subscription_handles
        }
    }
}

pub(crate) use sealed::ValidatedComposition;
