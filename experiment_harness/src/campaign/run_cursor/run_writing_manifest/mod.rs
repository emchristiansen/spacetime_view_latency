//! The initial run state — writing the once-per-run immutable manifest — and the root of the run cursor's
//! **linear nested typestate tree**.
//!
//! Each successor state is a child module of its exact predecessor: [`written_run`] and
//! [`run_seeding_background`] here, then `run_checking_initial_set`, then `run_dosing` (whose own child is
//! the awaiting-dose continuation). Every successor's constructor is `pub(super)`, confining it to that
//! predecessor's module subtree — so no module *outside* this nested tree (a `run_cursor` sibling, the run
//! driver in `campaign`, or the wider crate) can construct a successor to skip a transition or re-enter the
//! cursor mid-sequence. The whole tree is one **sealed implementation boundary**: the only entry from
//! outside is [`RunWritingManifest::begin`], and the only way through is the linear chain of
//! `pub(in crate::campaign)` transitions each state exposes, each consuming `self` and threading one owned
//! value. (Rust `pub(super)` also admits a module's own descendants, so this is a subtree-level seal, not a
//! claim that each constructor is callable by literally one module; within the subtree the states are
//! implemented to expose only their forward transitions.) The public cursor entities are re-exported upward
//! through each level and ultimately through `run_cursor`.
//!
//! Because each state's fields must be private *to its module* for the nesting to gate construction, the
//! types live in their `mod.rs` files rather than separate impl files: the nesting is what seals the
//! boundary against callers outside the subtree.

use crate::campaign::run_frontier::RunFrontier;
use crate::campaign::sink_write_stage::SinkWriteStage;
use crate::dataset::run_dataset::RunDataset;
use crate::observation::observation_sink::ObservationSink;

use super::RunExecutionStopped;
use super::RunManifestStep;

mod run_seeding_background;
mod written_run;

pub(crate) use run_seeding_background::RunAwaitingDose;
pub(crate) use run_seeding_background::RunDosing;
pub(crate) use run_seeding_background::RunSeedingBackground;

use written_run::WrittenRun;

/// A run that owns the sink but has not yet written its manifest. The only way to enter the pre-dose
/// preparation phases is [`Self::write_manifest`]: the warm-up phase ([`RunSeedingBackground`]) — and so
/// eventually dose 1 — is unreachable until the manifest write's contract returns success, so no effect or
/// observation can precede the manifest record.
pub(crate) struct RunWritingManifest {
    sink: ObservationSink,
}

impl RunWritingManifest {
    /// Begin a run in its manifest-writing state, taking ownership of the campaign's sink. The run's
    /// coordinate is *not* held here: it is derived from the manifest this state later writes
    /// ([`Self::write_manifest`]), so there is no second coordinate to drift from the manifest's own.
    /// `pub(in crate::campaign)` so only a block cursor starts a run.
    pub(in crate::campaign) fn begin(sink: ObservationSink) -> Self {
        Self { sink }
    }

    /// Recover the campaign's owned sink from the run's entry state, without writing anything. Used only
    /// when acquiring the run's resources fails — provisioning or
    /// connect — *before* the run's linear cleanup owner
    /// ([`RunCleanup`](crate::campaign::run_cleanup::RunCleanup)) could exist: the sink carrying prior
    /// runs' durable progress is threaded back out to be finalized into a
    /// [`CampaignAborted`](crate::campaign::campaign_aborted::CampaignAborted) rather than dropped. Only
    /// valid at this entry typestate, before any manifest or dose record — once a run has written, its
    /// cleanup owner exists and settlement, not abandonment, seals it. `pub(in crate::campaign)` so only
    /// the campaign orchestrator recovers the sink.
    pub(in crate::campaign) fn abandon(self) -> ObservationSink {
        self.sink
    }

    /// Write the run's manifest record, taking the bound run `context` (the [`RunDataset`] the driver
    /// resolved from this run's own manifest before cursor entry). The owned sink and context are handed to
    /// [`WrittenRun::write`], which performs the real write and — on success — returns them bound with the
    /// receipt as a single [`WrittenRun`]; the record written is `context.manifest()` and the run coordinate
    /// is read from that same manifest, so the manifest, its coordinate, the dataset, the receipt, and the
    /// sink carried onward are one owned value by construction, with no separately supplied coordinate,
    /// dataset, receipt, or sink to disagree. On success, advance to the warm-up phase
    /// ([`RunSeedingBackground`]); on a failure, [`WrittenRun::write`] returns the recovered sink with the
    /// [`PersistError`](crate::observation::persist_error::PersistError), and this stops at a
    /// [`RunExecutionStopped`] carrying a
    /// [`RunStage::WritingManifest`](crate::campaign::run_stage::RunStage::WritingManifest) frontier (built
    /// from the typed [`SinkWriteStage`] input) whose coordinate was read from that context *before* the
    /// write, plus the attempted record (if any) and the sink's retained poison. `pub(in crate::campaign)`
    /// so only the run driver advances the run.
    pub(in crate::campaign) fn write_manifest(self, context: RunDataset) -> RunManifestStep {
        // Derive the failure coordinate from the context *before* handing it to the write: `WrittenRun::write`
        // consumes the context and only returns it (bound to its own sink and receipt) on success, so on
        // failure the coordinate must already be in hand. The receipt/context/sink pairing happens entirely
        // inside `WrittenRun::write`, so nothing is supplied independently here.
        let coordinate = context.manifest().run_coordinate();
        match WrittenRun::write(self.sink, context) {
            Ok(written) => RunManifestStep::Seeding(RunSeedingBackground::new(written)),
            Err((sink, error)) => {
                let frontier = RunFrontier::stopped_by_sink_write(
                    coordinate,
                    SinkWriteStage::WritingManifest,
                    None,
                    None,
                    error.attempted_record(),
                    sink.poisoned().cloned(),
                );
                RunManifestStep::Stopped(RunExecutionStopped::new(sink, frontier))
            }
        }
    }
}
