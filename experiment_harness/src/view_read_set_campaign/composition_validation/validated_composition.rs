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
    use std::collections::HashMap;

    use anyhow::{bail, ensure, Context, Result};
    use serde::Serialize;

    use crate::view_read_set_campaign::campaign_params::OWNED_KEY_BASE;
    use crate::view_read_set_campaign::composition_validation::composition_transition_expectation::CompositionTransitionExpectation;
    use crate::view_read_set_campaign::composition_validation::expected_composition::ExpectedComposition;
    use crate::view_read_set_campaign::composition_validation::expected_foreign_visibility::ExpectedForeignVisibility;
    use crate::view_read_set_campaign::composition_validation::expected_payload_state::ExpectedPayloadState;
    use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
    use crate::view_read_set_campaign::scale_point::ScalePoint;

    /// How many rows of each preregistered slice one observation held.
    ///
    /// A named pair rather than a tuple because the two counts are the same type and swapping them
    /// would turn the Arm's passing security gate into its failure.
    struct SliceCensus {
        owned: u64,
        foreign: u64,
    }

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
    /// with the foreign slice seeded throughout. It then derives the final saturated write's own
    /// target from the bound schedule, requires that row to be present in both observations, and
    /// requires its final payload to differ from the one the *previous* write to that key left —
    /// since the pinned SpacetimeDB source elides a byte-identical update outright, an equal pair
    /// would mean the batch's last write measured nothing.
    ///
    /// **How the two arguments are bound to their phases — precisely.** Nothing compares the two
    /// [`ObservedRowSet`]s' paths or digests, so this is not an identity check, and handing the
    /// *same* artifact twice is not rejected as a duplicate. It is rejected by the phase
    /// expectations themselves: the before-census requires the seeded payload on every owned row and
    /// the after-census requires that key's derived final payload, and those two payloads differ
    /// bytewise by construction — the seeded constant and the mutation prefix are distinct frozen
    /// literals. With the owned slice fixed at ten rows and both censuses requiring exactly that
    /// count, no single artifact can satisfy both sides, so one passed twice always fails the second
    /// census. Swapping the two likewise fails. What is *not* established here is that either
    /// artifact came from the phase it is claimed for — only that it satisfies that phase's rule;
    /// which observation was taken when is the driver's measurement path, as genuineness always is.
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
    /// **Everything here is replayable, and that is why the live subscriber facts are not here.**
    /// Delivered rows, client-cache rows, and subscription handles were once inputs. Two of them
    /// cannot bear on whether these rows satisfy this transition, and all three were free
    /// parameters, so a caller could state any value and no comparison could contradict it — the
    /// same forgeability the caller-selected witness had. They also made a finding unmintable
    /// without a live server, which would have put this whole check beyond the reach of a test.
    /// Delivery and handle facts are supporting evidence about the subscriber's connection and are
    /// recorded *beside* the composition finding, on the attempt's evidence, not inside it.
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
    }

    /// The concrete witness to the **final saturated write** — the last measured mutation of the
    /// attempt — at the owned key that write targets.
    ///
    /// Private to this childless module, so it cannot be assembled independently of the check that
    /// produced it. It records concrete values rather than asserting that a change was seen, so
    /// "the final mutation was visible" is something a reader verifies against payloads instead of
    /// a flag they must believe.
    ///
    /// **Each field names its own provenance, because they do not share one.** The pre-image of the
    /// final write is *never observed*: capturing the state between two writes of a saturated batch
    /// would mean stopping the pipeline under measurement. It is instead exactly reconstructible
    /// from the frozen schedule, so it is recorded as an expectation and named as one. The other
    /// two really were validated against retained artifacts and are independently cross-checkable
    /// there. Giving a derived value and an observed value the same shape — `payload_before` beside
    /// `payload_after` — would have let a reader take the reconstruction for a measurement.
    ///
    /// The expected-immediate-before and validated-after pair is what witnesses *this* mutation.
    /// The seeded value is retained beside them because it is what makes the record checkable
    /// against the pre-E2 artifact, but seeded-versus-final alone would witness only that something
    /// changed at some point since seeding — not that the batch's last write landed.
    #[derive(Debug, Clone, Serialize)]
    struct MutationWitness {
        entity_key: u64,
        /// Validated against the pre-E2 artifact: the seeded payload this key carried before any
        /// measured write.
        seeded_payload_validated_before_e2: String,
        /// Derived from the bound after-phase schedule: the payload this key held immediately
        /// before the final saturated write replaced it. Not an observation.
        expected_payload_before_final_write: String,
        /// Validated against the post-E1 artifact: the payload this key carried once the saturated
        /// batch confirmed.
        payload_validated_after_e1: String,
    }

    impl ValidatedComposition {
        /// Check both observed result sets against their own side of `expected`, minting the
        /// artifact only if every applicable gate holds.
        ///
        /// **There is no witness parameter, and that is the contract.** The witness is *the final
        /// saturated write*, so which row it is at is a fact about the frozen schedule, not a choice
        /// a caller makes: it is derived here from the bound after-phase schedule's own
        /// [`final_write_offset`](crate::view_read_set_campaign::mutation_schedule::MutationSchedule::final_write_offset).
        /// An offset parameter — even a well-typed one confined to the owned slice — would have let
        /// a caller witness any of the ten owned keys, nine of which were last written earlier in
        /// the batch, so a finding could claim the final mutation while evidencing a different one.
        /// Witnessing another mutation is now unrepresentable rather than discouraged.
        ///
        /// The signature also fixes which observations a finding derives from: it takes the retained
        /// row sets rather than counts, so no caller can pre-reduce the evidence into something this
        /// function cannot contradict.
        pub(crate) fn validate(
            expected: CompositionTransitionExpectation,
            before: ObservedRowSet,
            after: ObservedRowSet,
        ) -> Result<Self> {
            let before_census = census(&before, expected.before())
                .context("censusing the observation retained before the first measured write")?;
            let after_census = census(&after, expected.after())
                .context("censusing the observation retained once the saturated batch confirmed")?;

            ensure!(
                before_census.owned == after_census.owned,
                "the measured identity's own slice held {} rows before measurement and {} after; \
                 the measured mutation updates in place, so cardinality is constant at every \
                 committed state",
                before_census.owned,
                after_census.owned,
            );

            let schedule = expected.after_schedule();
            let offset = schedule.final_write_offset();
            let entity_key = OWNED_KEY_BASE
                .checked_add(offset.get())
                .expect("an owned-slice offset names a key inside the frozen owned range");

            let seeded = payload_at(&before, entity_key).context(
                "reading the final saturated write's target from the pre-measurement observation",
            )?;
            let after_payload = payload_at(&after, entity_key)
                .context("reading the final saturated write's target from the final observation")?;
            let expected_before_final = schedule.penultimate_payload_at_offset(offset);

            ensure!(
                after_payload != expected_before_final,
                "the final saturated write's target entity_uuid={entity_key} ends the batch \
                 carrying the payload its *previous* write left there; a byte-identical update is \
                 elided at the pinned commit, so the final write measured nothing",
            );

            // The only allocation of witness payloads happens here, at the mint: the two validated
            // values are borrowed out of the retained artifacts for the checks above and owned only
            // once they become part of the record.
            let mutation = MutationWitness {
                entity_key,
                seeded_payload_validated_before_e2: seeded.to_string(),
                expected_payload_before_final_write: expected_before_final,
                payload_validated_after_e1: after_payload.to_string(),
            };

            Ok(Self {
                expected,
                before,
                after,
                observed_owned_rows: after_census.owned,
                observed_foreign_rows: after_census.foreign,
                mutation,
            })
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

        /// The owned key the final saturated write targeted — derived from the schedule, never
        /// chosen.
        pub(crate) fn witness_entity_key(&self) -> u64 {
            self.mutation.entity_key
        }

        /// The seeded payload that key carried in the pre-E2 artifact.
        pub(crate) fn witness_seeded_payload(&self) -> &str {
            &self.mutation.seeded_payload_validated_before_e2
        }

        /// The payload that key held immediately before the final saturated write, derived from the
        /// frozen schedule. Named as an expectation because no observation can hold it.
        pub(crate) fn witness_expected_payload_before_final_write(&self) -> &str {
            &self.mutation.expected_payload_before_final_write
        }

        /// The payload that key carried in the post-E1 artifact.
        pub(crate) fn witness_payload_after(&self) -> &str {
            &self.mutation.payload_validated_after_e1
        }

        /// How many rows the measured subscriber's client cache held once the saturated batch
        /// confirmed — **derived**, never stored.
        ///
        /// The after-census admits a row only by counting it into exactly one of the two
        /// preregistered ranges and rejects anything outside both, so these two counts partition the
        /// observation completely and their sum is its cardinality. Both the Arm's sender-scoped
        /// view and the Control's direct table are subscribed whole, so that cardinality *is* the
        /// cache's row count for the subscribed target.
        ///
        /// Deriving it is what keeps it honest. A second reading taken from the live cache could
        /// disagree with the retained artifact — a subscription update landing between the read that
        /// built the rows and the read that counted them is enough — and the disagreeing number
        /// would be the one recorded. Here there is only ever one observation to report.
        pub(crate) fn client_cache_rows(&self) -> u64 {
            self.observed_owned_rows
                .checked_add(self.observed_foreign_rows)
                .expect(
                    "both counts are cardinalities of one observation the census already accepted, \
                     so their sum is that observation's own row count and cannot overflow u64",
                )
        }
    }

    /// Check one observation against one phase's expectation, returning what each slice held.
    ///
    /// Every row must fall in one of the two preregistered key ranges and carry that range's
    /// expected owner and that phase's expected payload; a row outside both is rejected outright,
    /// because nothing else was ever seeded and an unexplained key means the observation is not the
    /// composition it claims to be. Counting rather than materializing the expected key set is
    /// sufficient because `entity_uuid` is the primary key, so distinct rows have distinct keys and
    /// range membership plus an exact count pins the set.
    fn census(observed: &ObservedRowSet, expected: &ExpectedComposition) -> Result<SliceCensus> {
        let (owned_start, owned_end) = expected.owned_key_range();
        let (foreign_start, foreign_end) = expected.foreign_key_range();

        // For the after phase, the expected payload differs per owned key, so the schedule's own
        // walk of the frozen slice is replayed once into an offset-keyed map. Building it from
        // `owned_slice_offsets` rather than from arithmetic on each key is what keeps the cycle
        // formula stated in exactly one place — the schedule.
        let final_payloads: Option<HashMap<u64, String>> = match expected.payload_state() {
            ExpectedPayloadState::Seeded => None,
            ExpectedPayloadState::AfterMeasuredBatch(schedule) => Some(
                schedule
                    .owned_slice_offsets()
                    .into_iter()
                    .map(|offset| (offset.get(), schedule.final_payload_at_offset(offset)))
                    .collect(),
            ),
        };

        let mut owned = 0u64;
        let mut foreign = 0u64;

        for row in observed.rows() {
            let owner_hex = row.owner.to_hex().to_string();

            if (owned_start..owned_end).contains(&row.entity_uuid) {
                ensure!(
                    owner_hex == expected.owned_owner_hex(),
                    "owned row entity_uuid={} is owned by {owner_hex}, not the measured identity \
                     its key range was seeded for",
                    row.entity_uuid,
                );
                let expected_payload: &str = match final_payloads.as_ref() {
                    None => expected.seeded_payload(),
                    Some(payloads) => {
                        let offset = row
                            .entity_uuid
                            .checked_sub(owned_start)
                            .expect("the row's key was just proven to be inside the owned range");
                        payloads.get(&offset).map(String::as_str).with_context(|| {
                            format!(
                                "owned row entity_uuid={} sits at slice offset {offset}, which the \
                                 frozen owned slice does not contain",
                                row.entity_uuid,
                            )
                        })?
                    }
                };
                ensure!(
                    row.record == expected_payload,
                    "owned row entity_uuid={} carries {:?}, not the {:?} this phase requires",
                    row.entity_uuid,
                    row.record,
                    expected_payload,
                );
                owned += 1;
            } else if (foreign_start..foreign_end).contains(&row.entity_uuid) {
                ensure!(
                    owner_hex == expected.foreign_owner_hex(),
                    "foreign row entity_uuid={} is owned by {owner_hex}, not the identity its key \
                     range was seeded for",
                    row.entity_uuid,
                );
                // No measured write ever targets the foreign slice, in any phase.
                ensure!(
                    row.record == expected.seeded_payload(),
                    "foreign row entity_uuid={} carries {:?} rather than the seeded payload; no \
                     measured write targets the foreign slice",
                    row.entity_uuid,
                    row.record,
                );
                foreign += 1;
            } else {
                bail!(
                    "row entity_uuid={} lies outside both preregistered key ranges [{owned_start}, \
                     {owned_end}) and [{foreign_start}, {foreign_end}); nothing else was ever seeded",
                    row.entity_uuid,
                );
            }
        }

        ensure!(
            owned == expected.owned_rows(),
            "the observation holds {owned} of the measured identity's {} own rows",
            expected.owned_rows(),
        );

        // A closed match, not a numeric threshold: `None` is the Arm's security gate, where one
        // leaked row is the candidate's answer rather than a small discrepancy.
        let expected_foreign = match expected.foreign_visibility() {
            ExpectedForeignVisibility::None => 0,
            ExpectedForeignVisibility::All { rows } => rows,
        };
        ensure!(
            foreign == expected_foreign,
            "the observation holds {foreign} foreign rows, expected {expected_foreign} for this \
             role; for the sender-scoped Arm any foreign row is a read-set leak",
        );

        Ok(SliceCensus { owned, foreign })
    }

    /// The payload one observation recorded at `key`, failing loud when the row is absent.
    ///
    /// Borrows out of the retained artifact rather than cloning: the comparisons this feeds need
    /// only a `&str`, so the record's single allocation happens where the witness is minted.
    fn payload_at(observed: &ObservedRowSet, key: u64) -> Result<&str> {
        let row = observed
            .rows()
            .iter()
            .find(|row| row.entity_uuid == key)
            .with_context(|| {
                format!("no row with entity_uuid={key} is present in the observation")
            })?;
        Ok(&row.record)
    }
}

pub(crate) use sealed::ValidatedComposition;

#[cfg(test)]
mod tests;
