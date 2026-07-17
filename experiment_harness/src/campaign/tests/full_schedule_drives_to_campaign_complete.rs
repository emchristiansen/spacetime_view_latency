//! Driving the whole seeded schedule with an always-succeeding writer reaches an affine `CampaignComplete`.

use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::dataset::dose_index::DoseIndex;
use crate::params::NUM_DOSES_USIZE;
use crate::plan::schedule::Schedule;

use super::drive;
use super::driving_writer::DrivingWriter;

/// The full 270-block / 540-run schedule, every run's manifest and ten dose observations written through
/// the real sink methods, drives to a clean finalize and mints [`CampaignComplete`]. Reaching completion
/// after exactly every scheduled block (asserted inside [`drive::drive_full_schedule`]) proves order and
/// exhaustion; the completion's typed progress reflects the last completed block down to its final dose.
#[test]
fn full_schedule_drives_to_campaign_complete() {
    // Pin the preregistered schedule size: 9 cells × 30 repetition blocks.
    assert_eq!(
        Schedule::preregistered()
            .randomized_block_order(drive::SEED)
            .len(),
        270,
        "the preregistered schedule is 9 cells × 30 repetition blocks"
    );

    let outcome = drive::drive_full_schedule(drive::SEED, DrivingWriter::always_ok());
    match outcome {
        CampaignOutcome::Complete(complete) => {
            assert_eq!(
                complete.seed(),
                drive::SEED,
                "the completion names the campaign seed"
            );
            let progress = complete.progress();
            assert!(
                progress.last_block().is_some(),
                "a completed campaign recorded its last block"
            );
            assert!(progress.last_run().is_some(), "and that block's last run");
            assert!(
                progress.last_successful_record().is_some(),
                "and a record whose writer contract returned success"
            );
            assert_eq!(
                progress.last_durable_dose(),
                Some(DoseIndex::ALL[NUM_DOSES_USIZE - 1]),
                "the last completed run finished its full ten-dose ladder"
            );
        }
        other => panic!("expected a complete campaign, got {other:?}"),
    }
}
