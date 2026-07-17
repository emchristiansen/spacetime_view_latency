//! A dose-write failure part-way through the ladder aborts up to an execution-failure outcome whose
//! frontier records the failing dose stage and the last record/dose whose writer contracts returned success.

use crate::campaign::campaign_incomplete::CampaignIncomplete;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::campaign::finalization_outcome::FinalizationOutcome;
use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunManifestStep;
use crate::campaign::run_cursor::RunObservationStep;
use crate::campaign::run_stage::RunStage;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::dose_observation::DoseObservation;

use super::drive;
use super::driving_writer::DrivingWriter;

/// The first run writes its manifest and the first two doses, then the third dose write fails. The run
/// stops at a [`RunStage::Dosing`] frontier for the third dose, retaining the last record and dose whose
/// writer contracts returned success (the second dose). The campaign aborts to
/// [`CampaignIncomplete::Execution`], finalizing in the same transition (`Failed`, since the failed dose
/// write poisoned the sink).
#[test]
fn dose_write_failure_yields_dosing_frontier() {
    // `ok_for(3)`: manifest + dose 1 + dose 2 succeed; the dose-3 write fails.
    let first = drive::open_first_run(drive::SEED, DrivingWriter::ok_for(3));
    let manifest = ValidatedRunManifest::fixture_for(first.coord.clone(), drive::SEED);
    let mut dosing = match first.writing.write_manifest(&manifest) {
        RunManifestStep::Dosing(dosing) => dosing,
        RunManifestStep::Incomplete(_) => panic!("the manifest write must succeed with three ok writes"),
    };

    let mut stopped = None;
    for (index, &dose) in DoseIndex::ALL.iter().enumerate() {
        let awaiting = match dosing.next_dose() {
            RunDoseStep::Awaiting(awaiting) => awaiting,
            RunDoseStep::Exhausted(_) => panic!("the ladder must not exhaust before the failing dose"),
        };
        let observation = DoseObservation::fixture_for(&manifest, dose);
        match awaiting.write_observation(&observation) {
            RunObservationStep::Dosing(next) => dosing = next,
            RunObservationStep::Incomplete(incomplete) => {
                assert_eq!(index, 2, "the third dose write (0-based index 2) is the one that fails");
                stopped = Some(incomplete);
                break;
            }
        }
    }
    let run_incomplete = stopped.expect("the scripted writer must fail the third dose write");

    let block_incomplete = first.run_resume.abort(run_incomplete);
    let outcome = first.block_resume.abort(block_incomplete);

    match outcome {
        CampaignOutcome::Incomplete(CampaignIncomplete::Execution {
            frontier,
            finalization,
        }) => {
            let block_frontier = frontier.block().expect("stopped inside a block");
            assert_eq!(block_frontier.block(), first.block);
            let run_frontier = block_frontier.run().expect("stopped inside a run");
            assert_eq!(run_frontier.run(), &first.coord);
            assert_eq!(
                run_frontier.stage(),
                RunStage::Dosing(DoseIndex::ALL[2]),
                "it stopped writing the third dose's observation"
            );
            assert!(
                run_frontier.last_successful_record().is_some(),
                "the manifest and first two dose writes returned success"
            );
            assert_eq!(
                run_frontier.last_durable_dose(),
                Some(DoseIndex::ALL[1]),
                "the second dose is the last whose observation write returned success"
            );
            match finalization {
                FinalizationOutcome::Failed(error) => assert!(
                    error.prior().is_some(),
                    "the dose-write failure poisoned the sink"
                ),
                FinalizationOutcome::Sealed => panic!("a poisoned sink cannot finalize clean"),
            }
        }
        other => panic!("expected an execution-failure incomplete campaign, got {other:?}"),
    }
}
