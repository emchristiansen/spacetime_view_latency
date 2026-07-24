//! `RunDataset::resolve` binds a manifest to the dataset resolved from that manifest's *own* coordinate: the
//! subscription target co-derives from the manifest, never from an independently supplied run.

use crate::dataset::subscribed_table::SubscribedTable;

use super::drive;

/// [`RunDataset::resolve`](crate::dataset::run_dataset::RunDataset::resolve) takes only a manifest (by value)
/// and the role identities — there is no independent run argument. So the bound context keeps the manifest's
/// own run coordinate, and its
/// subscription target co-derives from that same coordinate: it matches the target independently derived
/// from the run the manifest names. A mismatched manifest/dataset pair is unrepresentable by construction,
/// retiring the run-shape cross-check a separate manifest and dataset once needed.
#[test]
fn run_dataset_co_derives_dataset_from_owned_manifest() {
    let coord = drive::first_run_coordinate(drive::SEED);
    let context = drive::resolve_context(&coord, drive::SEED);

    assert_eq!(
        context.manifest().run_coordinate(),
        coord,
        "the bound context carries the manifest's own run coordinate"
    );
    assert_eq!(
        context.subscribed_table().subscription_sql(),
        SubscribedTable::from_run(coord.run()).subscription_sql(),
        "the subscription target co-derives from the manifest's own run, not an independent one"
    );
}
