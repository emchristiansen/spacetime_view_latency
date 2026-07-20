//! The report projection of one matched repetition block's raw arm/control observations.

use serde::Serialize;

use crate::analysis::report::run_report::RunReport;
use crate::analysis::validate::matched_block::MatchedBlock;
use crate::observation::record_seq::RecordSeq;

/// The report projection of one [`MatchedBlock`]: its schedule-proven collection-order key and the two
/// matched runs' raw observations — the arm under test and its matched direct-base-table control. The two
/// runs are distinct fields (not a same-typed pair), mirroring the trusted block's structural arm/control
/// distinction; the collection-order key is the exact [`RecordSeq`] the temporal diagnostics join on.
#[derive(Debug, Serialize)]
pub(crate) struct BlockReport {
    /// This block's position in the durable collection order (the earlier of its two runs' manifest
    /// sequences).
    collection_order_key: RecordSeq,
    /// The module-view arm run under test for this block.
    arm: RunReport,
    /// The matched direct-base-table control run for this block.
    control: RunReport,
}

impl BlockReport {
    /// Project one matched block's raw observations. One input — the trusted block — projected whole.
    pub(crate) fn of(block: &MatchedBlock) -> Self {
        // The two runs stay distinct fields, mirroring the trusted block's structural arm/control
        // distinction; the collection-order key is the exact sequence the temporal diagnostics join on.
        Self {
            collection_order_key: block.collection_order_key(),
            arm: RunReport::of(block.arm()),
            control: RunReport::of(block.control()),
        }
    }
}
