//! Schedule-order stage — positive proof: a campaign emitted in the true seed-derived schedule order
//! satisfies the stage-6 grammar binding and folds without a `ScheduleOrderMismatch`.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;

/// The fixture emits its records in the exact production schedule order — the seed-permuted global block
/// order, each block's seed-selected adjacent run order, and each run's manifest then canonical doses — so
/// the schedule-order stage binds every record to the grammar position it must hold and the complete
/// campaign folds. This isolates stage-6 acceptance of the *randomized* order; the ordering proofs
/// alongside each perturb this true order to fail exactly one grammar obligation.
#[test]
fn a_seed_ordered_campaign_folds_through_the_schedule_order() {
    ValidatedCampaign::from_records(CampaignFixture::valid().into_records())
        .expect("a campaign in true seed-derived schedule order must satisfy the schedule-order stage");
}
