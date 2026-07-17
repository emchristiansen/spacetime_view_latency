//! The run state between doses: holding the dose ladder iterator, ready to draw the next dose.

use std::array::IntoIter;

use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ManifestWriteReceipt;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_id::RecordId;
use crate::params::NUM_DOSES_USIZE;

use super::RunAwaitingDose;
use super::RunDoseStep;
use super::RunExecuted;

/// A run part-way through its dose ladder: it owns the sink, its coordinate, the manifest receipt its
/// observations key to, the remaining dose iterator, and the last record and dose whose writer contract
/// returned success. [`Self::next_dose`] draws the next dose from the iterator — advancing to
/// [`RunAwaitingDose`] — or, when the iterator is drained, advances to [`RunExecuted`] (the inert
/// exhausted-execution carrier the run's linear cleanup owner consumes). Progress is gated on the
/// iterator, never a count.
///
/// `last_record` is a required [`RecordId`], not an `Option`: this state is reachable only after the
/// manifest write's contract returned success, so a last successful record always exists here (it is the
/// manifest record until the first dose write's contract returns success). `last_dose` stays optional
/// because no dose has been written when the ladder begins.
pub(crate) struct RunDosing {
    sink: ObservationSink,
    coord: RunCoordinate,
    manifest: ManifestWriteReceipt,
    doses: IntoIter<DoseIndex, NUM_DOSES_USIZE>,
    last_record: RecordId,
    last_dose: Option<DoseIndex>,
}

impl RunDosing {
    /// Begin the dose ladder immediately after the manifest write. Seeds the full fixed ladder
    /// [`DoseIndex::ALL`] and records the manifest record as the last record whose contract returned
    /// success; no dose has been written yet.
    pub(in crate::campaign) fn begin(
        sink: ObservationSink,
        coord: RunCoordinate,
        manifest: ManifestWriteReceipt,
    ) -> Self {
        let last_record = manifest.record();
        Self {
            sink,
            coord,
            manifest,
            doses: DoseIndex::ALL.into_iter(),
            last_record,
            last_dose: None,
        }
    }

    /// Return to the between-doses state after a dose write's contract returned success, keeping the
    /// *same* partly-drained iterator so the ladder cannot be restarted or a dose repeated. `pub(super)`
    /// so only a run-cursor transition — not a campaign sibling — supplies the private dose iterator.
    pub(super) fn resume(
        sink: ObservationSink,
        coord: RunCoordinate,
        manifest: ManifestWriteReceipt,
        doses: IntoIter<DoseIndex, NUM_DOSES_USIZE>,
        last_record: RecordId,
        last_dose: Option<DoseIndex>,
    ) -> Self {
        Self {
            sink,
            coord,
            manifest,
            doses,
            last_record,
            last_dose,
        }
    }

    /// Draw the next dose. If the ladder iterator yields one, await its observation ([`RunAwaitingDose`]);
    /// when the iterator is drained, every dose's write contract has returned success, so advance to
    /// [`RunExecuted`] — dropping the manifest receipt, which is no longer needed once no more
    /// observations will be written.
    ///
    /// The `last_dose` `Option` is converted to a required [`DoseIndex`] exactly here, at the exhaustion
    /// edge, with a loud invariant check: the ten-dose ladder is nonempty and drains only by ten dose
    /// writes whose contracts returned success, so a last dose observation whose writer contract returned
    /// success always exists once the iterator is empty. Past this edge (`RunExecuted` onward) the
    /// marker is required, so `last_dose: None` is unrepresentable there.
    pub(in crate::campaign) fn next_dose(self) -> RunDoseStep {
        let mut doses = self.doses;
        match doses.next() {
            Some(active) => RunDoseStep::Awaiting(RunAwaitingDose::new(
                self.sink,
                self.coord,
                self.manifest,
                doses,
                self.last_record,
                self.last_dose,
                active,
            )),
            None => {
                let last_dose = self.last_dose.expect(
                    "the ten-dose ladder is nonempty, so exhaustion follows at least one dose observation whose writer contract returned success",
                );
                RunDoseStep::Exhausted(RunExecuted::new(
                    self.sink,
                    self.coord,
                    self.last_record,
                    last_dose,
                ))
            }
        }
    }
}
