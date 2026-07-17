//! A manifest-write failure on the first run aborts up to an execution-failure campaign outcome that
//! retains both the `WritingManifest` frontier and the always-attempted finalization.

use crate::campaign::campaign_incomplete::CampaignIncomplete;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::campaign::finalization_outcome::FinalizationOutcome;
use crate::campaign::run_stage::RunStage;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::campaign::run_cursor::RunManifestStep;

use super::drive;
use super::driving_writer::DrivingWriter;

/// The first run's manifest write fails (the writer's first line write fails). The run stops at a
/// [`RunStage::WritingManifest`] frontier with no record whose contract returned success, the block
/// lifts it, and the campaign aborts — always finalizing in the same transition. The outcome is
/// [`CampaignIncomplete::Execution`] retaining the exact frontier (seed, block, run, stage) *and* the
/// finalization: because the failed write poisoned the sink, that finalize is `Failed` with the prior
/// poison and no fresh sync failure (dual-failure preservation).
#[test]
fn manifest_write_failure_yields_execution_frontier() {
    // `ok_for(0)`: the very first line write — the manifest — fails.
    let first = drive::open_first_run(drive::SEED, DrivingWriter::ok_for(0));
    let manifest = ValidatedRunManifest::fixture_for(first.coord.clone(), drive::SEED);
    let run_incomplete = match first.writing.write_manifest(&manifest) {
        RunManifestStep::Incomplete(incomplete) => incomplete,
        RunManifestStep::Dosing(_) => panic!("ok_for(0) must fail the manifest write"),
    };
    let block_incomplete = first.run_resume.abort(run_incomplete);
    let outcome = first.block_resume.abort(block_incomplete);

    match outcome {
        CampaignOutcome::Incomplete(CampaignIncomplete::Execution {
            frontier,
            finalization,
        }) => {
            assert_eq!(frontier.seed(), drive::SEED, "the frontier names the campaign seed");
            let block_frontier = frontier.block().expect("this milestone stops inside a block");
            assert_eq!(block_frontier.block(), first.block, "at the first block's coordinate");
            let run_frontier = block_frontier.run().expect("and inside a run");
            assert_eq!(run_frontier.run(), &first.coord, "at the first run's coordinate");
            assert_eq!(
                run_frontier.stage(),
                RunStage::WritingManifest,
                "it stopped writing the manifest"
            );
            assert!(
                run_frontier.last_successful_record().is_none(),
                "no record's writer contract returned success before the manifest failed"
            );
            assert!(run_frontier.last_durable_dose().is_none(), "and no dose either");
            match finalization {
                FinalizationOutcome::Failed(error) => {
                    assert!(
                        error.prior().is_some(),
                        "the manifest-write failure poisoned the sink"
                    );
                    assert!(
                        error.sync_failures().is_none(),
                        "the writer's own final sync returned success"
                    );
                }
                FinalizationOutcome::Sealed => {
                    panic!("a poisoned sink cannot finalize clean")
                }
            }
        }
        other => panic!("expected an execution-failure incomplete campaign, got {other:?}"),
    }
}
