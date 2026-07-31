//! Which payload state a result set is expected to be observed in.

use serde::Serialize;

use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;

/// The phase of the measured schedule a composition expectation applies at.
///
/// **Why a phase and not a single payload.** The measured mutation deliberately changes payloads, so
/// "every row carries the one fixed payload" is true only *before* the first measured batch. Stating
/// it unconditionally is what made the composition contract contradict itself: it demanded the fixed
/// payload everywhere and simultaneously demanded that the mutated row's payload had changed. The
/// two claims are both correct — at different phases — so the phase is what the expectation names.
///
/// **What each case expects.**
///
/// - [`Self::Seeded`]: every row, owned and foreign, carries the seeded payload. The state before E2
///   issues its first measured write.
/// - [`Self::AfterMeasuredBatch`]: each owned key at slice offset `k` carries that channel's payload
///   for the last write to reach it, and every foreign row still carries the seeded payload. The
///   state after E2's or E1's batch has fully confirmed.
///
/// A channel whose batch never happened cannot be named: the variant carries a
/// [`MutationSchedule`], which exists only for the two channels that issue measured writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ExpectedPayloadState {
    /// Before any measured write: the seeded payload everywhere.
    Seeded,
    /// After this channel's whole measured batch confirmed: derived final payloads on the owned
    /// slice, seeded payloads still on the foreign slice.
    AfterMeasuredBatch(MutationSchedule),
}
