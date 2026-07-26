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
    /// The whole schedule is now derived: the channel projection, the two walks, and the three
    /// derivations. The update path they describe exists as the module's `update_entity_owner`
    /// reducer, reached through
    /// [`ConnectedClient::update_entity_owner`](crate::client::connected_client::ConnectedClient::update_entity_owner).
    /// Issuing the schedule against a live server is the driver's measurement stage, which is still
    /// a `todo!()`.
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

        /// The payload the owned key at `offset` holds once this channel's whole batch has
        /// confirmed.
        ///
        /// This is the derivable final state the composition check compares against: because the
        /// batch covers the slice a whole number of times — proven at compile time where the two
        /// constants are declared — the last write to reach offset `k` is index
        /// `CHANNEL_SAMPLE_COUNT - SUBSCRIBER_VISIBLE_ROWS_BASELINE + k`, which for the frozen
        /// constants is 990 through 999.
        pub(crate) fn final_payload_at_offset(self, offset: OwnedSliceOffset) -> String {
            self.payload(ChannelWriteIndex(
                CHANNEL_SAMPLE_COUNT - SUBSCRIBER_VISIBLE_ROWS_BASELINE + offset.get(),
            ))
        }
    }
}

// `ChannelWriteIndex` is deliberately not re-exported yet. Nothing outside this module names it —
// the two derivations that take one are reached through `writes()`, whose element type callers get
// by inference — and a re-export nothing uses is a warning, not a surface. Phase 2 adds it here the
// moment the driver spells the type out.
pub(crate) use sealed::{MutationSchedule, OwnedSliceOffset};

#[cfg(test)]
mod tests;
