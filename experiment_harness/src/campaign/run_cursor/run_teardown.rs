//! The final run state before completion: the isolated server's teardown transition.

use crate::campaign::teardown_reported::TeardownReported;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_id::RecordId;

use super::RunComplete;
use super::RunDone;

/// A run positioned at its server-teardown transition, past disconnect. Both last-success markers are
/// required here (`last_record: RecordId`, `last_dose: DoseIndex`), carried unchanged from
/// [`RunDisconnecting`](super::RunDisconnecting), so completion is minted from present values with no
/// re-check.
///
/// [`Self::teardown`] consumes [`TeardownReported`] evidence and is the *sole* minter of [`RunComplete`]:
/// a run cannot report completion without surrendering teardown evidence, which itself is reachable only
/// past the disconnect and the full dose ladder. The teardown *effect* belongs to the future measurement
/// driver; consuming the token advances the typestate only. When that driver wires the real effect, this
/// transition becomes fallible — returning a `Done`/`Incomplete` step whose failure arm builds a
/// `RunFrontier { stage: Teardown, last_successful_record: Some(last_record), last_durable_dose:
/// Some(last_dose), attempted_record: None, .. }` (a teardown is not a sink write).
pub(crate) struct RunTeardown {
    sink: ObservationSink,
    coord: RunCoordinate,
    last_record: RecordId,
    last_dose: DoseIndex,
}

impl RunTeardown {
    /// Position a run at its teardown transition after disconnect. `pub(in crate::campaign)` so only the
    /// disconnect transition builds one.
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

    /// Complete the run, consuming the driver's teardown-reported evidence. Mints the run's sole
    /// [`RunComplete`] from the present last-success markers and returns the sink for the block to
    /// recover. The evidence advances the typestate only; see [`TeardownReported`].
    pub(in crate::campaign) fn teardown(self, _reported: TeardownReported) -> RunDone {
        let complete = RunComplete::new(self.coord, self.last_record, self.last_dose);
        RunDone::new(self.sink, complete)
    }
}
