//! Shared driving helpers: step the consuming cursors in lockstep with an independently-replayed schedule.
//!
//! The cursor derives its schedule internally from the seed; these helpers replay the *same* seeded
//! schedule and build each run's fixture manifest/observations for the coordinate that schedule predicts.
//! Because every cursor transition binds the coordinate (`write_manifest`/`write_observation` assert the
//! fixture's coordinate equals the cursor's), a successful drive to completion *is* proof that the
//! cursor's internal schedule order matches the independent replay — a divergence would fail an assert.

use crate::campaign::block_cursor::BlockDone;
use crate::campaign::block_cursor::BlockReady;
use crate::campaign::block_cursor::BlockRunningRun;
use crate::campaign::block_cursor::BlockStep;
use crate::campaign::campaign_cursor::CampaignReady;
use crate::campaign::campaign_cursor::CampaignRunningBlock;
use crate::campaign::campaign_cursor::CampaignStep;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::campaign::run_cleanup;
use crate::campaign::run_cursor::RunDone;
use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunExecuted;
use crate::campaign::run_cursor::RunExecutionStopped;
use crate::campaign::run_cursor::RunManifestStep;
use crate::campaign::run_cursor::RunObservationStep;
use crate::campaign::run_cursor::RunSettled;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::dose_observation::DoseObservation;
use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::observation_sink::ObservationSink;
use crate::plan::schedule::BlockCoordinate;
use crate::plan::schedule::BlockRun;
use crate::plan::schedule::Schedule;

use super::driving_writer::DrivingWriter;

/// A deliberately non-zero campaign seed, so the tests exercise the actual seed threading (manifest
/// provenance, schedule permutation, arm/control order) rather than the degenerate `0`.
pub(super) const SEED: u64 = 0x5EED_C0DE;

/// Drive one run from its manifest-writing state to a completed [`RunDone`] using an always-succeeding
/// writer: write the manifest, write all ten dose observations in ladder order, then settle the exhausted
/// run through a no-I/O **reported** clean cleanup. The manifest and every observation are fixtures for
/// `coord` under `seed`, so each cursor coordinate/manifest/dose binding must match. The reported cleanup
/// performs the terminal typestate move only — it opens no socket and tears down no server (see
/// [`run_cleanup::settle_executed_reported`]).
pub(super) fn drive_run_to_completion(
    writing: RunWritingManifest,
    coord: &RunCoordinate,
    seed: u64,
) -> RunDone {
    let manifest = ValidatedRunManifest::fixture_for(coord.clone(), seed);
    let mut dosing = match writing.write_manifest(&manifest) {
        RunManifestStep::Seeding(seeding) => seeding.seeded().checked(),
        RunManifestStep::Stopped(_) => {
            panic!("the always-ok writer must let the manifest write succeed")
        }
    };
    for &dose in &DoseIndex::ALL {
        let awaiting = match dosing.next_dose() {
            RunDoseStep::Awaiting(awaiting) => awaiting,
            RunDoseStep::Exhausted(_) => {
                panic!("the ten-dose ladder must not exhaust before ten doses")
            }
        };
        let observation = DoseObservation::fixture_for(&manifest, dose);
        dosing = match awaiting.write_observation(&observation) {
            RunObservationStep::Dosing(dosing) => dosing,
            RunObservationStep::Stopped(_) => {
                panic!("the always-ok writer must let each dose write succeed")
            }
        };
    }
    let executed = match dosing.next_dose() {
        RunDoseStep::Exhausted(executed) => executed,
        RunDoseStep::Awaiting(_) => panic!("the ladder must exhaust after exactly ten doses"),
    };
    match run_cleanup::settle_executed_reported(executed) {
        RunSettled::Done(done) => done,
        RunSettled::Incomplete(_) => {
            panic!("a clean reported cleanup after full exhaustion completes the run")
        }
    }
}

/// Drive one block through its two runs to a completed [`BlockDone`], computing each run's coordinate
/// from the independently-replayed `[Run; 2]` order.
pub(super) fn drive_block(mut ready: BlockReady, block_run: &BlockRun, seed: u64) -> BlockDone {
    let runs = block_run.ordered_runs(seed);
    let mut run_idx = 0usize;
    loop {
        match ready.next_run() {
            BlockStep::Running { run, resume } => {
                let coord = RunCoordinate::new(block_run, runs[run_idx].role());
                let done = drive_run_to_completion(run, &coord, seed);
                ready = resume.finish(done);
                run_idx += 1;
            }
            BlockStep::Exhausted(done) => {
                assert_eq!(
                    run_idx, 2,
                    "a block runs exactly its two runs before exhausting"
                );
                return done;
            }
        }
    }
}

/// Drive the whole preregistered schedule to its terminal [`CampaignOutcome`] using `writer`: every
/// block, every run, every dose. Asserts the cursor drew every scheduled block before exhausting, then
/// returns the outcome of the mandatory finalize (`Complete` with an always-ok writer, or a
/// finalization-only `Incomplete` with a finalize-failing writer).
pub(super) fn drive_full_schedule(
    seed: u64,
    writer: impl DurableLineWriter + 'static,
) -> CampaignOutcome {
    let sink = ObservationSink::from_writer(Box::new(writer));
    let expected_blocks = Schedule::preregistered().randomized_block_order(seed);
    let mut cursor = CampaignReady::preregistered(seed, sink);
    let mut block_idx = 0usize;
    loop {
        match cursor.next_block() {
            CampaignStep::Running { block, resume } => {
                let block_run = &expected_blocks[block_idx];
                let done = drive_block(block, block_run, seed);
                cursor = resume.finish(done);
                block_idx += 1;
            }
            CampaignStep::Exhausted(exhausted) => {
                assert_eq!(
                    block_idx,
                    expected_blocks.len(),
                    "the cursor drew every scheduled block before exhausting"
                );
                return exhausted.finalize();
            }
        }
    }
}

/// The first run of the seeded schedule, opened to its manifest-writing state, plus the continuations
/// and coordinates a failure test needs to abort it back up to a campaign outcome.
pub(super) struct FirstRun {
    /// The first run in its manifest-writing state.
    pub(super) writing: RunWritingManifest,
    /// The block's continuation while this run is outstanding.
    pub(super) run_resume: BlockRunningRun,
    /// The campaign's continuation while this block is outstanding.
    pub(super) block_resume: CampaignRunningBlock,
    /// The first run's exact coordinate (for building its fixture and asserting its frontier).
    pub(super) coord: RunCoordinate,
    /// The first block's exact coordinate (for asserting its frontier).
    pub(super) block: BlockCoordinate,
}

/// Open the seeded schedule's first block and first run, threading `writer` into the sink, and hand back
/// the run's manifest-writing state with both continuations. Used by failure tests that must fail a write
/// and abort back up to a [`CampaignOutcome`].
pub(super) fn open_first_run(seed: u64, writer: impl DurableLineWriter + 'static) -> FirstRun {
    let sink = ObservationSink::from_writer(Box::new(writer));
    let blocks = Schedule::preregistered().randomized_block_order(seed);
    let first_block = &blocks[0];
    let runs = first_block.ordered_runs(seed);
    let coord = RunCoordinate::new(first_block, runs[0].role());
    let block = BlockCoordinate::of(first_block);

    let (block_ready, block_resume) = match CampaignReady::preregistered(seed, sink).next_block() {
        CampaignStep::Running { block, resume } => (block, resume),
        CampaignStep::Exhausted(_) => panic!("the preregistered schedule is nonempty"),
    };
    let (writing, run_resume) = match block_ready.next_run() {
        BlockStep::Running { run, resume } => (run, resume),
        BlockStep::Exhausted(_) => panic!("a fresh block has two runs"),
    };
    FirstRun {
        writing,
        run_resume,
        block_resume,
        coord,
        block,
    }
}

/// Drive the seeded schedule's first run through the whole ten-dose ladder to its inert
/// exhausted-execution carrier [`RunExecuted`], stopping *before* cleanup. Identical to
/// [`drive_run_to_completion`] up to ladder exhaustion, but hands back the pre-cleanup carrier itself
/// rather than settling it — so a cleanup-settlement test can apply a *synthetic*
/// [`RunCleanupOutcome`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome) to a genuinely
/// exhausted run. There is no production path that fabricates a `RunExecuted`; this drives the real
/// cursor through the always-ok no-I/O writer. `pub(in crate::campaign)` (test-only) so the sibling
/// `run_cleanup` test tree — which is not a `campaign::tests` descendant — can reach it.
pub(in crate::campaign) fn drive_first_run_to_executed() -> RunExecuted {
    let first = open_first_run(SEED, DrivingWriter::always_ok());
    let manifest = ValidatedRunManifest::fixture_for(first.coord.clone(), SEED);
    let mut dosing = match first.writing.write_manifest(&manifest) {
        RunManifestStep::Seeding(seeding) => seeding.seeded().checked(),
        RunManifestStep::Stopped(_) => {
            panic!("the always-ok writer must let the manifest write succeed")
        }
    };
    for &dose in &DoseIndex::ALL {
        let awaiting = match dosing.next_dose() {
            RunDoseStep::Awaiting(awaiting) => awaiting,
            RunDoseStep::Exhausted(_) => {
                panic!("the ten-dose ladder must not exhaust before ten doses")
            }
        };
        let observation = DoseObservation::fixture_for(&manifest, dose);
        dosing = match awaiting.write_observation(&observation) {
            RunObservationStep::Dosing(dosing) => dosing,
            RunObservationStep::Stopped(_) => {
                panic!("the always-ok writer must let each dose write succeed")
            }
        };
    }
    match dosing.next_dose() {
        RunDoseStep::Exhausted(executed) => executed,
        RunDoseStep::Awaiting(_) => panic!("the ladder must exhaust after exactly ten doses"),
    }
}

/// Drive the seeded schedule's first run to an inert [`RunExecutionStopped`] by failing its very first
/// sink write — the manifest record — with a writer scripted to fail immediately (`ok_for(0)`). The
/// stopped carrier holds a [`RunStage::WritingManifest`](crate::campaign::run_stage::RunStage::WritingManifest)
/// execution frontier, so a cleanup-settlement test can apply a synthetic failed cleanup to a genuinely
/// stopped run and prove both the execution frontier and the cleanup failure are retained.
/// `pub(in crate::campaign)` (test-only), like [`drive_first_run_to_executed`].
pub(in crate::campaign) fn drive_first_run_to_stopped() -> RunExecutionStopped {
    let first = open_first_run(SEED, DrivingWriter::ok_for(0));
    let manifest = ValidatedRunManifest::fixture_for(first.coord.clone(), SEED);
    match first.writing.write_manifest(&manifest) {
        RunManifestStep::Stopped(stopped) => stopped,
        RunManifestStep::Seeding(_) => panic!("ok_for(0) must fail the manifest write"),
    }
}
