//! Shared helpers driving one run's *real* run cursor to an inert pre-cleanup carrier.
//!
//! These drive the run cursor's own linear typestate chain — `RunWritingManifest::begin → write_manifest →
//! seeded → checked → the ten-dose ladder` — through the no-I/O scripted writer, with no block/campaign
//! [`Step`](crate::campaign::block_cursor::BlockStep) seam and no scripted continuation carrier: the block
//! and campaign carriers now own their run/block folding privately, reachable only through the real effect
//! path. What survives as a no-I/O unit concern is the run cursor itself — the manifest/dose write path and
//! the pre-cleanup [`RunExecuted`]/[`RunExecutionStopped`] carriers the sibling `run_cleanup` settlement
//! tests settle. The run context ([`RunDataset`]) is resolved from the run's own fixture manifest, so the
//! dataset co-derives from that manifest rather than an independently supplied run.

use spacetimedb_sdk::Identity;

use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunExecuted;
use crate::campaign::run_cursor::RunExecutionStopped;
use crate::campaign::run_cursor::RunManifestStep;
use crate::campaign::run_cursor::RunObservationStep;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::run_dataset::RunDataset;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::dose_evidence::DoseEvidence;
use crate::observation::observation_sink::ObservationSink;
use crate::plan::schedule::Schedule;
use crate::roles::role_identities::RoleIdentities;

use super::driving_writer::DrivingWriter;

/// A deliberately non-zero campaign seed, so the tests exercise the actual seed threading (manifest
/// provenance, schedule permutation, arm/control order) rather than the degenerate `0`. The one typed
/// [`ScheduleSeed`] the tests carry unchanged through every seeded API — schedule ordering, manifest
/// provenance/fixtures, and identity resolution — so no parallel raw `u64` is threaded alongside it.
pub(super) const SEED: ScheduleSeed = ScheduleSeed::new(0x5EED_C0DE);

/// The seeded schedule's first run coordinate: the first block of the randomized order, at that block's
/// first (arm-or-control) ordered run. Derived independently of any cursor, so a test can name the exact run
/// the run cursor's fixtures and frontier assertions key to.
pub(super) fn first_run_coordinate(seed: ScheduleSeed) -> RunCoordinate {
    let blocks = Schedule::preregistered().randomized_block_order(seed);
    let first_block = &blocks[0];
    let runs = first_block.ordered_runs(seed);
    RunCoordinate::new(first_block, runs[0].role())
}

/// Resolve the bound run context ([`RunDataset`]) for `coord` under `seed`: the run's fixture manifest bound
/// to the dataset [`RunDataset::resolve`] derives from that manifest's own coordinate. The measured identity
/// is a fixture stand-in for the server-issued connection identity (mirroring
/// [`DoseObservation::fixture_for`](crate::observation::dose_observation::DoseObservation)); the growth
/// identity is derived and must differ from it. Because `resolve` takes no independent run, the dataset
/// co-derives from the manifest by construction.
pub(super) fn resolve_context(coord: &RunCoordinate, seed: ScheduleSeed) -> RunDataset {
    let manifest = ValidatedRunManifest::fixture_for(coord.clone(), seed);
    let measured = Identity::from_claims("view-read-set-experiment-fixture", "fixture-measured");
    let identities = RoleIdentities::resolve(measured, seed, coord.cell())
        .expect("the fixture measured and growth identities are distinct");
    RunDataset::resolve(manifest, &identities)
}

/// Drive the seeded schedule's first run through the whole ten-dose ladder to its inert
/// exhausted-execution carrier [`RunExecuted`], stopping *before* cleanup. Writes the manifest and then each
/// dose's observation — assembled by the cursor from its *own* owned context and drawn active dose, from the
/// [`DoseEvidence`] fixture — through the always-ok no-I/O writer, then hands back the pre-cleanup carrier
/// itself rather than settling it, so a cleanup-settlement test can apply a *synthetic*
/// [`RunCleanupOutcome`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome) to a genuinely exhausted
/// run. There is no production path that fabricates a `RunExecuted`; this drives the real cursor.
/// `pub(in crate::campaign)` (test-only) so the sibling `run_cleanup` test tree — which is not a
/// `campaign::tests` descendant — can reach it.
pub(in crate::campaign) fn drive_first_run_to_executed() -> RunExecuted {
    let coord = first_run_coordinate(SEED);
    let context = resolve_context(&coord, SEED);
    let sink = ObservationSink::from_writer(Box::new(DrivingWriter::always_ok()));
    let mut dosing = match RunWritingManifest::begin(sink).write_manifest(context) {
        RunManifestStep::Seeding(seeding) => seeding.seeded().checked(),
        RunManifestStep::Stopped(_) => {
            panic!("the always-ok writer must let the manifest write succeed")
        }
    };
    for _ in &DoseIndex::ALL {
        let awaiting = match dosing.next_dose() {
            RunDoseStep::Awaiting(awaiting) => awaiting,
            RunDoseStep::Exhausted(_) => {
                panic!("the ten-dose ladder must not exhaust before ten doses")
            }
        };
        // The observation is assembled inside `write_observation` from the awaiting state's own owned
        // context and drawn active dose; the test supplies only the measured evidence.
        dosing = match awaiting.write_observation(DoseEvidence::fixture()) {
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
    let coord = first_run_coordinate(SEED);
    let context = resolve_context(&coord, SEED);
    let sink = ObservationSink::from_writer(Box::new(DrivingWriter::ok_for(0)));
    match RunWritingManifest::begin(sink).write_manifest(context) {
        RunManifestStep::Stopped(stopped) => stopped,
        RunManifestStep::Seeding(_) => panic!("ok_for(0) must fail the manifest write"),
    }
}
