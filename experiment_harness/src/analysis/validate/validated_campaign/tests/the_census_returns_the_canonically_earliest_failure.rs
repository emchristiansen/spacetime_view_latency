//! First-error-order proof: the stage-5 census traverses the canonical cell→role→block→dose lattice, not
//! the input record order, so with two dose-census failures arranged in reverse sequence order it returns
//! the one at the earliest canonical slot.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::dose_census_fault::DoseCensusFault;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::params::NUM_DOSES;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// The canonically-earliest missing dose: cell0/arm/block0's dose 4, ahead of every other run in the
/// census traversal.
const EARLY_MISSING_DOSE: u64 = 4;
/// The canonically-later missing dose: cell0/control/block0's dose 8. Its run is visited after the entire
/// arm role, so it is canonically later than [`EARLY_MISSING_DOSE`] despite being placed first in the
/// reversed record sequence.
const LATE_MISSING_DOSE: u64 = 8;

/// Dropping one dose from cell0/arm/block0 (canonically first) and one from cell0/control/block0
/// (canonically later), then reversing the record order so the control run is folded first, leaves two
/// dose-census failures whose sequence order is the reverse of their canonical order. The census counts
/// per canonical slot and reports the first canonical hole, so the fold returns the arm run's missing
/// dose — proving the traversal follows the canonical lattice, not the input sequence.
#[test]
fn the_census_returns_the_canonically_earliest_failure() {
    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    // cell0/block0 layout: arm manifest at 0, its dose d at index d; control manifest at NUM_DOSES+1, its
    // dose d at index (NUM_DOSES+1)+d. Remove the higher index first so the lower index stays valid.
    let early_index = EARLY_MISSING_DOSE as usize;
    let late_index = (NUM_DOSES as usize + 1) + LATE_MISSING_DOSE as usize;
    records.remove(late_index);
    records.remove(early_index);
    // Reverse the record order so the canonically-later control failure is folded ahead of the
    // canonically-earlier arm failure; renumber restores contiguous sequences over the reversed order.
    records.reverse();
    fixture.renumber();

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a campaign missing two doses must fail total validation");
    };

    match error {
        IntegrityError::DoseCensus {
            run,
            fault:
                DoseCensusFault::Missing {
                    dose,
                    expected_occurrences,
                    observed_occurrences,
                },
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the census returns the canonically-earliest run, not the earlier-sequenced control run"
            );
            assert_eq!(
                dose, EARLY_MISSING_DOSE,
                "the returned dose is the arm run's hole, not the control run's later canonical hole"
            );
            assert_eq!(expected_occurrences, 1, "each run must carry each dose exactly once");
            assert_eq!(observed_occurrences, 0, "the arm run's dose 4 observation is absent");
        }
        other => panic!("expected the canonically-earliest DoseCensus Missing, got {other:?}"),
    }
}
