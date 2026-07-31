//! The wire traffic one measured window received, as exact integers.

/// How many frames and how many bytes the subscriber's connection received over one window.
///
/// A named pair rather than two loose `u64`s: the two counts are the same type, so a positional
/// constructor would let a swap record a frame count as a byte count with nothing left to contradict
/// it — the reason
/// [`SliceCensus`](crate::view_read_set_campaign::composition_validation::validated_composition)
/// is a named pair too. The fields are visible to this module alone and written by exactly one
/// producer, [`WireCounterSnapshot::since`](super::wire_counter_snapshot::WireCounterSnapshot::since),
/// which mints one only after every arithmetic gate the window must pass has passed. There is no
/// constructor from loose numbers, so an unchecked delta is not a value that exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReceivedWireDelivery {
    /// Websocket frames received over the window, of every kind the connection carried.
    pub(in crate::view_read_set_campaign::subscriber_delivery) frames: u64,
    /// Compressed on-the-wire bytes those frames carried.
    pub(in crate::view_read_set_campaign::subscriber_delivery) bytes: u64,
}

impl ReceivedWireDelivery {
    /// Frames received over the window.
    pub(crate) fn frames(self) -> u64 {
        self.frames
    }

    /// Compressed on-the-wire bytes received over the window.
    pub(crate) fn bytes(self) -> u64 {
        self.bytes
    }
}
