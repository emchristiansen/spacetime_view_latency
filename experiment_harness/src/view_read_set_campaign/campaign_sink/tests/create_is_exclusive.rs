//! A second create on the same path fails exclusively, leaving the first ledger in place.

use crate::observation::output_path::OutputPath;
use crate::view_read_set_campaign::campaign_sink::CampaignSink;

use super::fixture;
use super::scratch_dir::scratch_dir;

/// Coverage: that a rerun cannot truncate a prior run's evidence. This is `O_EXCL` at the
/// [`FileLineWriter`](crate::observation::file_line_writer::FileLineWriter) below, asserted here at
/// the surface the campaign actually calls, because the guarantee that matters is "a second campaign
/// pointed at an existing ledger fails before writing anything," not that one layer down uses a
/// particular flag.
///
/// The first ledger is given a real record before the second attempt, so what the second create is
/// refused against is a file with evidence in it rather than an empty placeholder — and the file is
/// asserted afterwards to still hold that line, since a create that failed *after* truncating would
/// also return an error.
#[test]
fn create_is_exclusive() {
    let dir = scratch_dir("create-is-exclusive");
    let path = dir.join("campaign.ndjson");
    let output = OutputPath::new(path.clone());

    let mut first = CampaignSink::create(&output).expect("the first exclusive create succeeds");
    first
        .append(fixture::inventory_record())
        .expect("the opening inventory line persists");
    // Release the first sink's handles cleanly before the second attempt.
    first
        .finalize()
        .expect("finalizing the first ledger cleanly");

    let before = std::fs::read(&path).expect("the first ledger is readable");
    assert!(
        !before.is_empty(),
        "the first ledger must hold the line it appended"
    );

    // Matched rather than `expect_err`, which would require the sink itself to be `Debug`: it holds
    // a boxed writer, and deriving one solely to phrase this assertion would put a rendering of the
    // ledger's private state within reach of every caller.
    let refused = match CampaignSink::create(&output) {
        Err(refused) => format!("{refused:#}"),
        Ok(_) => panic!("a second exclusive create on an existing path must not succeed"),
    };
    assert!(
        refused.contains("creating the campaign ledger"),
        "the refusal must name what it was creating, got {refused:?}"
    );

    let after = std::fs::read(&path).expect("the first ledger is still readable");
    assert_eq!(
        after, before,
        "a refused create must leave the existing ledger byte-identical"
    );
}
