//! A fully-executed schedule whose only failure is the mandatory finalize yields a finalization-only
//! incomplete that retains the typed latest progress.

use crate::campaign::campaign_incomplete::CampaignIncomplete;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::dataset::dose_index::DoseIndex;
use crate::params::NUM_DOSES_USIZE;

use super::drive;
use super::driving_writer::DrivingWriter;

/// With every write succeeding but the sink's finalize scripted to fail, driving the whole schedule
/// executes every block and then fails only at the mandatory finalize. The outcome is
/// [`CampaignIncomplete::Finalization`], not `Complete`: it retains the typed high-water progress (down
/// to the last completed run's final dose) and the sink's typed [`FinalizeError`], with no execution
/// frontier and no prior poison — the failure is the final sync alone.
#[test]
fn finalization_only_failure_retains_typed_progress() {
    let outcome = drive::drive_full_schedule(drive::SEED, DrivingWriter::finalize_failing());
    match outcome {
        CampaignOutcome::Incomplete(CampaignIncomplete::Finalization { progress, error }) => {
            assert!(
                progress.last_block().is_some(),
                "execution completed, so the typed progress is retained"
            );
            assert_eq!(
                progress.last_durable_dose(),
                Some(DoseIndex::ALL[NUM_DOSES_USIZE - 1]),
                "the last completed run finished its full ten-dose ladder"
            );
            assert!(
                error.sync_failures().is_some(),
                "the scripted finalize failed the file/directory sync"
            );
            assert!(
                error.prior().is_none(),
                "no write failed, so the sink carried no prior poison into finalize"
            );
            assert!(
                error.is_durability_ambiguous(),
                "a final-sync failure leaves the durable tail ambiguous"
            );
        }
        other => panic!("expected a finalization-only incomplete campaign, got {other:?}"),
    }
}
