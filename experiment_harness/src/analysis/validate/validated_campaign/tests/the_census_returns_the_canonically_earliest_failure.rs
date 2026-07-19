//! First-error-order proof: the stage-5 census traverses the canonical cell→role→block→dose lattice, not
//! the input record order, so with two dose-census failures arranged in reverse sequence order it returns
//! the one at the earliest canonical slot.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::dose_census_fault::DoseCensusFault;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
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
    // Locate the two canonical holes by identity: cell0/arm/block0's dose 4 (canonically first) and
    // cell0/control/block0's dose 8 (canonically later). Under the seed record order either run may sit
    // first, so remove the higher record index first to keep the lower one valid.
    let early_dose = DoseIndex::ALL[(EARLY_MISSING_DOSE - 1) as usize];
    let late_dose = DoseIndex::ALL[(LATE_MISSING_DOSE - 1) as usize];
    let early_index = fixture.dose_index(Cell::all()[0], RunRole::Arm, 0, early_dose);
    let late_index = fixture.dose_index(Cell::all()[0], RunRole::Control, 0, late_dose);
    let records = fixture.records_mut();
    let (higher, lower) = if early_index > late_index {
        (early_index, late_index)
    } else {
        (late_index, early_index)
    };
    records.remove(higher);
    records.remove(lower);
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
