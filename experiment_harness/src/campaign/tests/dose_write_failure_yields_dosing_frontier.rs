//! A dose-write failure part-way through the ladder stops the run at a `Dosing` execution frontier that
//! records the failing dose stage and the last record/dose whose writer contracts returned success.

use crate::campaign::run_cleanup;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;
use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunManifestStep;
use crate::campaign::run_cursor::RunObservationStep;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::campaign::run_incompletion::RunIncompletion;
use crate::campaign::run_stage::RunStage;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::dose_evidence::DoseEvidence;
use crate::observation::observation_sink::ObservationSink;

use super::drive;
use super::driving_writer::DrivingWriter;

/// The first run writes its manifest and the first two doses — each observation assembled by the cursor from
/// its *own* owned context and drawn active dose — then the third dose write fails. The run stops at a
/// [`RunStage::Dosing`] frontier for the third dose, retaining the last record and dose whose writer
/// contracts returned success (the second dose). A reported clean cleanup settles it into a
/// [`RunIncompletion::Execution`] whose frontier is keyed to the bound context's own run.
///
/// The successful writes of doses one and two are themselves positive proof that
/// [`RunAwaitingDose::write_observation`](crate::campaign::run_cursor::RunAwaitingDose) derives each
/// observation from the owned context and binds it to the run's own manifest — a foreign observation is
/// unrepresentable, so it needs no rejection test.
#[test]
fn dose_write_failure_yields_dosing_frontier() {
    let coord = drive::first_run_coordinate(drive::SEED);
    let context = drive::resolve_context(&coord, drive::SEED);
    // `ok_for(3)`: manifest + dose 1 + dose 2 succeed; the dose-3 write fails.
    let sink = ObservationSink::from_writer(Box::new(DrivingWriter::ok_for(3)));
    let mut dosing = match RunWritingManifest::begin(sink).write_manifest(context) {
        RunManifestStep::Seeding(seeding) => seeding.seeded().checked(),
        RunManifestStep::Stopped(_) => {
            panic!("the manifest write must succeed with three ok writes")
        }
    };

    let mut stopped_execution = None;
    for (index, _dose) in DoseIndex::ALL.iter().enumerate() {
        let awaiting = match dosing.next_dose() {
            RunDoseStep::Awaiting(awaiting) => awaiting,
            RunDoseStep::Exhausted(_) => {
                panic!("the ladder must not exhaust before the failing dose")
            }
        };
        // The observation is assembled inside `write_observation` from the awaiting state's own context and
        // drawn dose; the test supplies only the measured evidence.
        match awaiting.write_observation(DoseEvidence::fixture()) {
            RunObservationStep::Dosing(next) => dosing = next,
            RunObservationStep::Stopped(stopped) => {
                assert_eq!(
                    index, 2,
                    "the third dose write (0-based index 2) is the one that fails"
                );
                stopped_execution = Some(stopped);
                break;
            }
        }
    }
    let stopped = stopped_execution.expect("the scripted writer must fail the third dose write");

    // The run's linear cleanup owner mints the RunIncomplete; a no-I/O reported *clean* cleanup settles it
    // into an `Execution` incompletion carrying the execution frontier and a `Clean` cleanup outcome.
    let (_sink, incompletion) = run_cleanup::settle_stopped_reported(stopped).into_parts();
    let frontier = match incompletion {
        RunIncompletion::Execution { frontier, cleanup } => {
            assert!(
                matches!(cleanup, RunCleanupOutcome::Clean),
                "the reported cleanup after the execution failure is clean"
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
        RunStage::Dosing(DoseIndex::ALL[2]),
        "it stopped writing the third dose's observation"
    );
    assert!(
        frontier.last_successful_record().is_some(),
        "the manifest and first two dose writes returned success"
    );
    assert_eq!(
        frontier.last_durable_dose(),
        Some(DoseIndex::ALL[1]),
        "the second dose is the last whose observation write returned success"
    );
}
