//! Two rows sharing a primary key are refused, and nothing is written.

use crate::view_read_set_campaign::campaign_params::SEEDED_ROW_SET_STEM;
use crate::view_read_set_campaign::composition_validation::observed_row_set::{
    ObservedRowSet, ROW_SET_EXTENSION,
};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

use super::attempt_directory::attempt_directory;
use super::row::row;

/// Coverage: `entity_uuid` is the table's primary key, so two rows sharing one is a state no server
/// held — but this API takes an ordinary row collection, so a faulty cache snapshot or a duplicated
/// read-back can hand one over anyway. Accepting it would give an impossible table state a retained
/// artifact and a digest, which is precisely the shape of evidence that later gets believed.
///
/// It would also make the content address depend on arrival order: with equal keys the sort has a
/// tie, so two encodings of the same multiset could differ in which duplicate came first. That is
/// the property the ordering exists to erase.
///
/// The check runs through persistence, because the contract is that the rejection happens *before*
/// anything is written — so the test also requires that no artifact appeared. A rejection that left
/// a partial file behind would leave an unreferenced artifact in the attempt's directory, and the
/// phase's one exclusive creation already spent.
#[test]
fn a_duplicate_entity_uuid_is_rejected() {
    let directory = attempt_directory("duplicate-key");

    let duplicated = vec![
        row(7, "view-read-set-campaign-seeded-payload"),
        row(3, "view-read-set-campaign-seeded-payload"),
        row(
            7,
            "view-read-set-campaign-mutation:e1-saturated-queue-growth:997",
        ),
    ];

    let error = ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        duplicated,
    )
    .expect_err("two rows sharing a primary key must not acquire an artifact");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("entity_uuid=7"),
        "the failure must name the duplicated key, got {rendered}"
    );

    let path = directory
        .path()
        .join(format!("{SEEDED_ROW_SET_STEM}{ROW_SET_EXTENSION}"));
    assert!(
        !path.exists(),
        "the rejection must happen before persistence, leaving no artifact at {}",
        path.display()
    );
}
