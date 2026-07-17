//! The run state after the dose ladder is drained, before the measured client's disconnect transition.

use crate::campaign::disconnect_reported::DisconnectReported;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_id::RecordId;

use super::RunTeardown;

/// A run whose dose ladder is fully drained, positioned at its disconnect transition. Reachable only
/// from [`RunDosing::next_dose`](super::RunDosing) exhaustion, so every dose write's contract returned
/// success by the time this state exists. The manifest receipt has been dropped — no further
/// observations will be written — leaving only the last-success markers to carry into completion.
///
/// Both markers are required here — `last_record: RecordId` (a manifest write's contract returned
/// success) and `last_dose: DoseIndex` (all ten dose writes' contracts returned success) — because this
/// state is past the ladder-exhaustion edge where the accumulating `last_dose` `Option` is converted
/// once with a loud invariant check. `last_dose: None` is unrepresentable from here on.
///
/// [`Self::disconnect`] consumes [`DisconnectReported`] evidence to advance to [`RunTeardown`]. The
/// disconnect *effect* belongs to the future measurement driver; consuming the token advances the
/// typestate only and asserts nothing about the socket itself. When that driver wires the real effect,
/// this transition becomes fallible — returning a `Teardown`/`Incomplete` step whose failure arm builds
/// a `RunFrontier { stage: Disconnecting, last_successful_record: Some(last_record), last_durable_dose:
/// Some(last_dose), attempted_record: None, .. }` (a disconnect is not a sink write, so it has no
/// in-flight record). The required markers here map straight into those `Some(..)` frontier slots.
pub(crate) struct RunDisconnecting {
    sink: ObservationSink,
    coord: RunCoordinate,
    last_record: RecordId,
    last_dose: DoseIndex,
}

impl RunDisconnecting {
    /// Position a run at its disconnect transition after ladder exhaustion. `pub(in crate::campaign)` so
    /// only the dose cursor's exhaustion transition builds one.
    pub(in crate::campaign) fn new(
        sink: ObservationSink,
        coord: RunCoordinate,
        last_record: RecordId,
        last_dose: DoseIndex,
    ) -> Self {
        Self {
            sink,
            coord,
            last_record,
            last_dose,
        }
    }

    /// Advance past the disconnect transition, consuming the driver's disconnect-reported evidence. The
    /// evidence advances the typestate only; see [`DisconnectReported`].
    pub(in crate::campaign) fn disconnect(self, _reported: DisconnectReported) -> RunTeardown {
        RunTeardown::new(self.sink, self.coord, self.last_record, self.last_dose)
    }
}
