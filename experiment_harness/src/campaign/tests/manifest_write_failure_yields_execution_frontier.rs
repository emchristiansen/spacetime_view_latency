//! A manifest-write failure stops the run at a `WritingManifest` execution frontier keyed to the run's own
//! bound-context coordinate, which its linear cleanup owner settles into an execution incompletion.

use crate::campaign::run_cleanup;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;
use crate::campaign::run_cursor::RunManifestStep;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::campaign::run_incompletion::RunIncompletion;
use crate::campaign::run_stage::RunStage;
use crate::observation::observation_sink::ObservationSink;

use super::drive;
use super::driving_writer::DrivingWriter;

/// The first run's manifest write fails (the writer's first line write fails). The run stops at a
/// [`RunStage::WritingManifest`] frontier whose coordinate is the one the bound context's *own* manifest
/// determines — there is no separately supplied coordinate to drift from it — with no record or dose yet
/// durable. A reported clean cleanup settles the inert stopped carrier into a
/// [`RunIncompletion::Execution`] retaining that exact frontier.
///
/// This is the run-cursor slice of the old campaign-level abort test: the block/campaign abort-and-finalize
/// fold is no longer a no-I/O unit concern, because the paired block and campaign carriers own it privately
/// and reach it only through the real effect path.
#[test]
fn manifest_write_failure_yields_execution_frontier() {
    let coord = drive::first_run_coordinate(drive::SEED);
    let context = drive::resolve_context(&coord, drive::SEED);
    // `ok_for(0)`: the very first line write — the manifest — fails.
    let sink = ObservationSink::from_writer(Box::new(DrivingWriter::ok_for(0)));
    let stopped = match RunWritingManifest::begin(sink).write_manifest(context) {
        RunManifestStep::Stopped(stopped) => stopped,
        RunManifestStep::Seeding(_) => panic!("ok_for(0) must fail the manifest write"),
    };

    // The failing edge yields only the inert stopped-execution carrier; a no-I/O reported *clean* cleanup
    // settles it into the RunIncomplete carrying the execution frontier.
    let (_sink, incompletion) = run_cleanup::settle_stopped_reported(stopped).into_parts();
    let frontier = match incompletion {
        RunIncompletion::Execution { frontier, cleanup } => {
            assert!(
                matches!(cleanup, RunCleanupOutcome::Clean),
                "the reported cleanup after the manifest-write failure is clean"
            );
            frontier
        }
        RunIncompletion::Cleanup { .. } => {
            panic!("execution failed, so this is an Execution incompletion, not Cleanup")
        }
    };
    assert_eq!(
        frontier.run(),
        &coord,
        "the frontier's coordinate is the bound context's own run"
    );
    assert_eq!(
        frontier.stage(),
        RunStage::WritingManifest,
        "it stopped writing the manifest"
    );
    assert!(
        frontier.last_successful_record().is_none(),
        "no record's writer contract returned success before the manifest failed"
    );
    assert!(frontier.last_durable_dose().is_none(), "and no dose either");
}
