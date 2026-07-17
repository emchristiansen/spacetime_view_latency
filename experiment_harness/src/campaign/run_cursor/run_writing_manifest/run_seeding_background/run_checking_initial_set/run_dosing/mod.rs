//! The between-doses run state and its nested awaiting-dose continuation.
//!
//! This module owns the dose cycle as a real ownership boundary, not a convention. [`RunDosing`] holds all
//! cycle state (sink, bound context, dose iterator, and last-success markers) in **private** fields, and
//! the only state that can pause mid-dose — [`RunAwaitingDose`] — is a **child module** of this one. A child
//! module can read and write its parent module's private fields, so `RunAwaitingDose` performs the real
//! observation write through the *nested* `RunDosing`'s own sink, consumes the affine
//! [`DoseWriteReceipt`](crate::observation::observation_sink::DoseWriteReceipt) internally, updates that
//! same nested `RunDosing`'s private progress, and returns it — without any `RunDosing` method that a
//! run-cursor sibling could call to write a foreign observation through one run's dosing or to rebuild the
//! cycle from loose fields. There is no `resume` and no cross-sibling progress accessor: the cycle advances
//! only by a real write against its own sink, and `RunAwaitingDose` is minted by [`RunDosing::next_dose`],
//! which draws the active dose from the dosing's own iterator.
//!
//! Because `RunDosing`'s fields must be private *to this module* for the child to reach them, the type and
//! its impl live here in `mod.rs` rather than in a separate impl file: the nesting is what makes the
//! boundary hold.

use std::array::IntoIter;

use crate::campaign::run_cursor::run_writing_manifest::written_run::WrittenRun;
use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunExecuted;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::run_dataset::RunDataset;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_id::RecordId;
use crate::params::NUM_DOSES_USIZE;

mod run_awaiting_dose;

pub(crate) use run_awaiting_dose::RunAwaitingDose;

/// A run part-way through its dose ladder: it owns the sink, its bound context (manifest + resolved
/// dataset), the remaining dose iterator, and the last record and dose whose writer contract returned
/// success. [`Self::next_dose`] draws the next dose from the iterator — advancing to [`RunAwaitingDose`] —
/// or, when the iterator is drained, advances to [`RunExecuted`] (the inert exhausted-execution carrier the
/// run's linear cleanup owner consumes). Progress is gated on the iterator, never a count.
///
/// **Every field is private to this module** so that only the nested [`RunAwaitingDose`] continuation — a
/// child module — can touch cycle state, and only by performing a real write against this dosing's own
/// `sink`. There is no field-taking constructor other than [`Self::begin`] (which consumes an unforgeable
/// [`WrittenRun`] into a fresh ladder), so no method reachable from outside this module's subtree can
/// restart, reorder, or advance the cycle from a foreign run's progress.
///
/// The manifest write receipt is *not* held here: it seeded `last_record` at [`Self::begin`] and provided
/// the pre-dose stop frontiers' last-successful record, both of which are past. From here the
/// durable-progress marker is the evolving `last_record`, so the receipt was dropped when the [`WrittenRun`]
/// was consumed into the owned sink and context. Observations key to `context.manifest()`, not the receipt.
///
/// `last_record` is a required [`RecordId`], not an `Option`: this state is reachable only after the
/// manifest write's contract returned success, so a last successful record always exists here (it is the
/// manifest record until the first dose write's contract returns success). `last_dose` stays optional
/// because no dose has been written when the ladder begins.
pub(crate) struct RunDosing {
    sink: ObservationSink,
    context: RunDataset,
    doses: IntoIter<DoseIndex, NUM_DOSES_USIZE>,
    last_record: RecordId,
    last_dose: Option<DoseIndex>,
}

impl RunDosing {
    /// Begin the dose ladder immediately after the initial-set check, consuming the bound [`WrittenRun`].
    /// Seeds the full fixed ladder [`DoseIndex::ALL`] and records the manifest record (read from the
    /// [`WrittenRun`]'s own receipt) as the last record whose contract returned success; no dose has been
    /// written yet. The receipt has now served its purpose, so the bundle is consumed into its owned sink and
    /// context alone — the run coordinate is thereafter read from `context.manifest()`, never a separate
    /// field, and the sink is never paired anew with a separately held context. `pub(super)` — where `super`
    /// is the [`RunCheckingInitialSet`](super::RunCheckingInitialSet) module — so within the sealed cursor
    /// subtree its `checked` transition is the call site that mints one; nothing outside the subtree can.
    pub(super) fn begin(written: WrittenRun) -> Self {
        let last_record = written.receipt().record();
        let (sink, context) = written.into_parts();
        Self {
            sink,
            context,
            doses: DoseIndex::ALL.into_iter(),
            last_record,
            last_dose: None,
        }
    }

    /// Draw the next dose. If the ladder iterator yields one, pair *this* dosing with that drawn dose in a
    /// [`RunAwaitingDose`] — the awaited dose is always the one this dosing's own iterator yielded. When the
    /// iterator is drained, every dose's write contract has returned success, so advance to [`RunExecuted`],
    /// reading the terminal coordinate from the owned context before it is dropped.
    ///
    /// The `last_dose` `Option` is converted to a required [`DoseIndex`] exactly here, at the exhaustion
    /// edge, with a loud invariant check: the ten-dose ladder is nonempty and drains only by ten dose
    /// writes whose contracts returned success, so a last dose observation whose writer contract returned
    /// success always exists once the iterator is empty. Past this edge (`RunExecuted` onward) the
    /// marker is required, so `last_dose: None` is unrepresentable there.
    pub(in crate::campaign) fn next_dose(mut self) -> RunDoseStep {
        match self.doses.next() {
            Some(active) => RunDoseStep::Awaiting(RunAwaitingDose::new(self, active)),
            None => {
                let last_dose = self.last_dose.expect(
                    "the ten-dose ladder is nonempty, so exhaustion follows at least one dose observation whose writer contract returned success",
                );
                // Read the terminal coordinate from the owned bound context before it is dropped: the
                // exhausted carrier needs only the coordinate and last-success markers, so the dataset is
                // released here.
                let coord = self.context.manifest().run_coordinate();
                RunDoseStep::Exhausted(RunExecuted::new(
                    self.sink,
                    coord,
                    self.last_record,
                    last_dose,
                ))
            }
        }
    }
}
