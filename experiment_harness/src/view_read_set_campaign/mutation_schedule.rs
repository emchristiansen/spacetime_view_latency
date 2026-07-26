//! The deterministic write schedule one measured channel applies to the owned slice.

use serde::Serialize;

use crate::view_read_set_campaign::campaign_params::{
    CHANNEL_SAMPLE_COUNT, SUBSCRIBER_VISIBLE_ROWS_BASELINE,
};
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;

mod channel_write_index;
mod owned_slice_offset;

pub(crate) use channel_write_index::ChannelWriteIndex;
pub(crate) use owned_slice_offset::OwnedSliceOffset;

/// The frozen target-and-payload schedule of a channel that issues measured writes.
///
/// **What the schedule is.** For zero-based write index `i`, the measured write targets
/// `OWNED_KEY_BASE + (i mod SUBSCRIBER_VISIBLE_ROWS_BASELINE)` and carries the payload
/// `<MUTATION_PAYLOAD_PREFIX>:<channel tag>:<i>`. Targeting the measured identity's own slice is what
/// keeps cardinality fixed while measuring and keeps the mutated row inside the Arm's read set,
/// which E2's "stopping when the change is observable in the subscriber cache" requires. Every write
/// differs bytewise from the state it replaces, so none is elided at the pinned commit.
///
/// **Why it is a type and not three loose functions.** Only two of the four channels issue measured
/// writes at all. [`Self::of`] is a total match that returns `None` for the two apply channels, so a
/// schedule exists precisely where writes do — asking for E3's target key is not a question that can
/// be posed. That in turn makes
/// [`ExpectedPayloadState::AfterMeasuredBatch`](super::composition_validation::expected_payload_state::ExpectedPayloadState::AfterMeasuredBatch)
/// unable to name a channel whose batch never happened.
///
/// **Why the derivations take minted indices.** [`ChannelWriteIndex`] and [`OwnedSliceOffset`] are
/// declared in this module's own children, so only this module can construct them — and it does so
/// only by walking the frozen batch length and the frozen slice. A `u64` parameter would let a
/// caller ask for write 5,000's target or offset 99's final payload, and the modulus arithmetic
/// would return a well-formed answer to a question about a write that is never issued and a row that
/// does not exist.
///
/// **Phase 1 boundary.** The channel projection and the two walks are complete; the three
/// derivations are explicit `todo!()` stubs, and no reducer capable of performing an update exists
/// yet.
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
        (0..CHANNEL_SAMPLE_COUNT)
            .map(ChannelWriteIndex::at)
            .collect()
    }

    /// Every offset of the measured identity's owned slice, ascending — the only offsets in
    /// existence. This is what the after-state expectation walks to name each key's final payload.
    pub(crate) fn owned_slice_offsets(self) -> Vec<OwnedSliceOffset> {
        (0..SUBSCRIBER_VISIBLE_ROWS_BASELINE)
            .map(OwnedSliceOffset::at)
            .collect()
    }

    /// The `entity_uuid` this measured write updates.
    pub(crate) fn target_key(self, write: ChannelWriteIndex) -> u64 {
        let _ = write;
        todo!(
            "Phase 2: OWNED_KEY_BASE + (write index mod SUBSCRIBER_VISIBLE_ROWS_BASELINE), so the \
             batch cycles the owned slice and cardinality never changes"
        )
    }

    /// The payload this measured write writes.
    pub(crate) fn payload(self, write: ChannelWriteIndex) -> String {
        let _ = write;
        todo!(
            "Phase 2: format MUTATION_PAYLOAD_PREFIX, this schedule's tag, and the write index as \
             <prefix>:<tag>:<index>, so no write repeats a payload and no channel collides with the \
             other at its batch boundary"
        )
    }

    /// The payload the owned key at `offset` holds once this channel's whole batch has confirmed.
    ///
    /// This is the derivable final state the composition check compares against: because the batch
    /// covers the slice a whole number of times — proven at compile time where the two constants are
    /// declared — the last write to reach offset `k` is index
    /// `CHANNEL_SAMPLE_COUNT - SUBSCRIBER_VISIBLE_ROWS_BASELINE + k`, which for the frozen constants
    /// is 990 through 999.
    pub(crate) fn final_payload_at_offset(self, offset: OwnedSliceOffset) -> String {
        let _ = offset;
        todo!(
            "Phase 2: this schedule's payload at write index CHANNEL_SAMPLE_COUNT - \
             SUBSCRIBER_VISIBLE_ROWS_BASELINE + offset — the last write of the batch to touch that \
             key"
        )
    }
}
