//! Category proof: dropping every record for one cell fails the stage-5 cell census with a typed
//! distinct-set contradiction naming the missing cell.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::cell_census_fault::CellCensusFault;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::plan::cell::Cell;

/// Removing an entire cell's manifests and observations together leaves the binding consistent but the
/// census short one preregistered cell, so the fold fails with a typed `CellCensus` whose distinct-set
/// evidence is the canonical expected set against an observed set missing exactly the dropped cell.
#[test]
fn a_missing_cell_fails_the_cell_census() {
    let dropped = Cell::all()[0];

    let mut fixture = CampaignFixture::valid();
    fixture
        .records_mut()
        .retain(|record| CampaignFixture::record_cell(record) != dropped);
    fixture.renumber();

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a campaign missing an entire cell must fail total validation");
    };

    match error {
        IntegrityError::CellCensus {
            fault: CellCensusFault::DistinctSet { expected, observed },
            ..
        } => {
            assert_eq!(
                expected,
                Cell::all(),
                "the expected set is the nine preregistered cells"
            );
            assert_eq!(
                observed.len(),
                Cell::all().len() - 1,
                "exactly one cell is absent from the census"
            );
            assert!(
                !observed.contains(&dropped),
                "the dropped cell is the one absent from the census"
            );
        }
        other => panic!("expected CellCensus DistinctSet, got {other:?}"),
    }
}
