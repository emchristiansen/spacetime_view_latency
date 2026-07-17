//! A monotonic global sequence number assigned to each durable sink record.

use serde::Serialize;

/// The campaign-global position of a record in the durable NDJSON stream, assigned by the
/// [`ObservationSink`](super::observation_sink::ObservationSink) in write order. It lets an
/// incomplete-run frontier name the last record whose durable write returned success and the record
/// whose write was in flight, without inspecting file contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct RecordSeq(u64);

impl RecordSeq {
    /// The sequence assigned to the first record written to a fresh sink.
    pub(crate) fn zero() -> Self {
        Self(0)
    }

    /// The next sequence after this one. Checked so exhausting `u64` fails loud and identically in
    /// debug and release rather than wrapping; the finite campaign never approaches this bound.
    pub(crate) fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("record sequence must not overflow u64"),
        )
    }

    /// The raw sequence value.
    pub(crate) fn get(self) -> u64 {
        self.0
    }
}
