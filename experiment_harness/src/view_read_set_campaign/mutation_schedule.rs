//! The deterministic write schedule one measured channel applies to the owned slice.
//!
//! **Module topology is the enforcement mechanism here, not a comment.** The schedule and its two
//! index types live in the private, *childless* inline module [`sealed`], and both index types'
//! fields are private to it. Rust makes a private field visible to its declaring module **and every
//! descendant**, so declaring the indices in child modules — or beside the schedule in a file that
//! later grows a child — leaves them mintable by code that never walked a frozen bound. Giving
//! `sealed` no children makes the set of code that can mint an index exactly the set that walks
//! `0..CHANNEL_SAMPLE_COUNT` or `0..SUBSCRIBER_VISIBLE_ROWS_BASELINE`, enforced by the compiler and
//! stable under anything added to this file afterwards.

mod sealed {
    use serde::Serialize;

    use crate::view_read_set_campaign::campaign_params::{
        CHANNEL_SAMPLE_COUNT, MUTATION_PAYLOAD_PREFIX, OWNED_KEY_BASE,
        SUBSCRIBER_VISIBLE_ROWS_BASELINE,
    };
    use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;

    /// The 0-based index of one measured write within a channel's frozen batch.
    ///
    /// Proves batch membership only, and structurally: the field is private to this childless
    /// module and there is no constructor, so [`MutationSchedule::writes`] is the only expression in
    /// the crate that can build one, and it walks exactly `0..CHANNEL_SAMPLE_COUNT`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    #[serde(transparent)]
    pub(crate) struct ChannelWriteIndex(u64);

    impl ChannelWriteIndex {
        /// The 0-based write number within its channel's batch.
        pub(crate) fn get(self) -> u64 {
            self.0
        }
    }

    /// The 0-based offset of one key within the measured identity's frozen owned slice.
    ///
    /// Confined exactly as [`ChannelWriteIndex`] is, and by the same mechanism: the sole minting
    /// expression is the struct literal in [`MutationSchedule::owned_slice_offsets`], which walks
    /// `0..SUBSCRIBER_VISIBLE_ROWS_BASELINE`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    #[serde(transparent)]
    pub(crate) struct OwnedSliceOffset(u64);

    impl OwnedSliceOffset {
        /// The 0-based offset within the owned slice.
        pub(crate) fn get(self) -> u64 {
            self.0
        }
    }

    /// The frozen target-and-payload schedule of a channel that issues measured writes.
    ///
    /// **What the schedule is.** For zero-based write index `i`, the measured write targets
    /// `OWNED_KEY_BASE + (i mod SUBSCRIBER_VISIBLE_ROWS_BASELINE)` and carries the payload
    /// `<MUTATION_PAYLOAD_PREFIX>:<channel tag>:<i>`. Targeting the measured identity's own slice is
    /// what keeps cardinality fixed while measuring and keeps the mutated row inside the Arm's read
    /// set, which E2's "stopping when the change is observable in the subscriber cache" requires.
    /// Every write differs bytewise from the state it replaces, so none is elided at the pinned
    /// commit.
    ///
    /// **Why it is a type and not three loose functions.** Only two of the four channels issue
    /// measured writes at all. [`Self::of`] is a total match that returns `None` for the two apply
    /// channels, so a schedule exists precisely where writes do — asking for E3's target key is not
    /// a question that can be posed. That in turn makes
    /// [`ExpectedPayloadState::AfterMeasuredBatch`](crate::view_read_set_campaign::composition_validation::expected_payload_state::ExpectedPayloadState::AfterMeasuredBatch)
    /// unable to name a channel whose batch never happened.
    ///
    /// **Why the derivations take minted indices.** [`ChannelWriteIndex`] and [`OwnedSliceOffset`]
    /// can only come from [`Self::writes`] and [`Self::owned_slice_offsets`], which walk the frozen
    /// batch length and the frozen slice and nothing else. A `u64` parameter would let a caller ask
    /// for write 5,000's target or offset 99's final payload, and the modulus arithmetic would
    /// return a well-formed answer to a question about a write that is never issued and a row that
    /// does not exist.
    ///
    /// The whole schedule is now derived: the channel projection, the two walks, the two payload
    /// derivations, and the final write's own offset and pre-image. The update path they describe
    /// exists as the module's `update_entity_owner`
    /// reducer, reached through
    /// [`ConnectedClient::update_entity_owner`](crate::client::connected_client::ConnectedClient::update_entity_owner).
    /// The driver's measurement stage now issues the schedule against a live server, walking
    /// [`Self::writes`] once per write-issuing channel — the paced channel one confirmed sample at a
    /// time, the saturated channel back-to-back.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    pub(crate) struct MutationSchedule {
        channel: MeasurementChannel,
        tag: &'static str,
    }

    impl MutationSchedule {
        /// This channel's measured-write schedule, or `None` when it issues no measured writes.
        ///
        /// Derived from [`MeasurementChannel::mutation_tag`] rather than from a second match, so the
        /// set of write-issuing channels is stated in exactly one place.
        pub(crate) fn of(channel: MeasurementChannel) -> Option<Self> {
            channel.mutation_tag().map(|tag| Self { channel, tag })
        }

        /// Which channel issues these writes.
        pub(crate) fn channel(self) -> MeasurementChannel {
            self.channel
        }

        /// The stable payload tag distinguishing this channel's writes from the other channel's.
        pub(crate) fn tag(self) -> &'static str {
            self.tag
        }

        /// Every measured write of this channel's batch, in issue order — the only write indices in
        /// existence. Consuming this in order is how the driver walks the batch.
        pub(crate) fn writes(self) -> Vec<ChannelWriteIndex> {
            (0..CHANNEL_SAMPLE_COUNT).map(ChannelWriteIndex).collect()
        }

        /// Every offset of the measured identity's owned slice, ascending — the only offsets in
        /// existence. This is what the after-state expectation walks to name each key's final
        /// payload.
        pub(crate) fn owned_slice_offsets(self) -> Vec<OwnedSliceOffset> {
            (0..SUBSCRIBER_VISIBLE_ROWS_BASELINE)
                .map(OwnedSliceOffset)
                .collect()
        }

        /// The `entity_uuid` this measured write updates.
        ///
        /// Cycling the owned slice is what holds cardinality fixed: every target is a key the
        /// attempt already seeded, so the write replaces a row rather than adding one. The modulus
        /// cannot escape the slice, and the addition cannot overflow — the compile-time proof at
        /// [`crate::view_read_set_campaign::campaign_params`] keeps
        /// `OWNED_KEY_BASE + SUBSCRIBER_VISIBLE_ROWS_BASELINE` at or below `GLOBAL_KEY_BASE`, which
        /// is itself far below `u64::MAX`.
        pub(crate) fn target_key(self, write: ChannelWriteIndex) -> u64 {
            OWNED_KEY_BASE + (write.get() % SUBSCRIBER_VISIBLE_ROWS_BASELINE)
        }

        /// The payload this measured write writes.
        ///
        /// Distinct for every `(channel, write index)` pair, which is what makes the write a real
        /// one: the pinned source elides a byte-identical update outright, so a repeated payload
        /// would measure nothing. The tag is what keeps E1's write `i` from reproducing the payload
        /// E2 already left on that same key.
        pub(crate) fn payload(self, write: ChannelWriteIndex) -> String {
            format!("{MUTATION_PAYLOAD_PREFIX}:{}:{}", self.tag, write.get())
        }

        /// The owned-slice offset this channel's *last* measured write targets.
        ///
        /// Derived from the frozen batch length rather than named as a literal, and returned as an
        /// [`OwnedSliceOffset`] so it is interchangeable with the offsets
        /// [`Self::owned_slice_offsets`] yields. For the frozen constants the last write is index
        /// 999 and this is offset 9.
        ///
        /// This is what makes the final-mutation witness unforgeable by a caller: the offset the
        /// witness is taken at comes from the schedule itself, so "witness a different row" is not
        /// a request that can be made.
        pub(crate) fn final_write_offset(self) -> OwnedSliceOffset {
            let last = CHANNEL_SAMPLE_COUNT
                .checked_sub(1)
                .expect("a measured batch issues at least one write");
            OwnedSliceOffset(last % SUBSCRIBER_VISIBLE_ROWS_BASELINE)
        }

        /// The index of the last measured write to reach `offset`.
        ///
        /// Shared by the two payload derivations below so the cycle arithmetic is written once: a
        /// second copy could drift, and the two payloads it produces are compared against each
        /// other by the composition check, where a drift would read as a real mutation.
        ///
        /// Because the batch covers the slice a whole number of times — proven at compile time
        /// where the two constants are declared — that index is
        /// `CHANNEL_SAMPLE_COUNT - SUBSCRIBER_VISIBLE_ROWS_BASELINE + k`, which for the frozen
        /// constants is 990 through 999. The subtraction is checked despite that proof: this
        /// derivation is read far from where the premise is written, and a drift in either constant
        /// must fail loud here rather than wrap into a plausible index.
        fn final_write_index(offset: OwnedSliceOffset) -> u64 {
            CHANNEL_SAMPLE_COUNT
                .checked_sub(SUBSCRIBER_VISIBLE_ROWS_BASELINE)
                .expect("a measured batch covers the owned slice at least once")
                .checked_add(offset.get())
                .expect("the last write to an owned-slice offset has an index within the batch")
        }

        /// The payload the owned key at `offset` holds once this channel's whole batch has
        /// confirmed.
        ///
        /// This is the derivable final state the composition check compares against.
        pub(crate) fn final_payload_at_offset(self, offset: OwnedSliceOffset) -> String {
            self.payload(ChannelWriteIndex(Self::final_write_index(offset)))
        }

        /// The payload the owned key at `offset` held *immediately before* this channel's last
        /// write to it — the pre-image of the final measured mutation at that key.
        ///
        /// Writes reach an offset once per cycle, so the write before index
        /// `CHANNEL_SAMPLE_COUNT - SUBSCRIBER_VISIBLE_ROWS_BASELINE + k` is one whole slice earlier;
        /// for the frozen constants the final write 999 at offset 9 is preceded by write 989.
        ///
        /// **Why this is derived and not observed.** No retained observation can hold it: capturing
        /// the state between two writes of a saturated batch would mean stopping the pipeline that
        /// is under measurement. It is exactly reconstructible from the frozen schedule instead,
        /// which is why the witness records it as an *expectation* rather than as an observation.
        /// Pairing it with the validated after-state is what witnesses the final mutation
        /// specifically, rather than merely witnessing that something changed since seeding.
        ///
        /// The subtraction is checked against the compile-time premise that a batch covers the
        /// slice at least twice, for the same reason [`Self::final_write_index`] is.
        pub(crate) fn penultimate_payload_at_offset(self, offset: OwnedSliceOffset) -> String {
            let previous = Self::final_write_index(offset)
                .checked_sub(SUBSCRIBER_VISIBLE_ROWS_BASELINE)
                .expect(
                    "a measured batch covers the owned slice at least twice, so the final write to \
                     an offset has a predecessor in its own batch",
                );
            self.payload(ChannelWriteIndex(previous))
        }
    }
}

// Neither index type is re-exported. Nothing outside this module names either one: the derivations
// that take an index are reached through `writes()` and `owned_slice_offsets()`, whose element types
// callers get by inference, and the composition check derives the final write's offset through
// `final_write_offset()` rather than spelling the type. A re-export nothing uses is a warning, not a
// surface; each is added here the moment a caller genuinely names it.
pub(crate) use sealed::MutationSchedule;

#[cfg(test)]
mod tests;
