//! `StockObservabilityReport::all` projects exactly the closed limit set, in its fixed order — a dropped,
//! added, or reordered limit fails here.

use crate::analysis::report::stock_observability_limit::{
    STOCK_OBSERVABILITY_LIMIT_COUNT, StockObservabilityLimit,
};
use crate::analysis::report::stock_observability_report::StockObservabilityReport;

#[test]
fn all_projects_the_closed_limit_set_in_fixed_order() {
    let report = StockObservabilityReport::all();

    // The boxed fixed array makes the two-limit cardinality a property of the type; tie it to the single
    // frozen count so a change to the stated set must move in lockstep.
    assert_eq!(report.limits.len(), STOCK_OBSERVABILITY_LIMIT_COUNT);

    // The exact ordered variant set: both which limits are stated and in which order.
    assert_eq!(
        *report.limits,
        [
            StockObservabilityLimit::PerViewReadSetClassNotDirectlyObservable,
            StockObservabilityLimit::PerViewMaterializerOperationCountsNotDirectlyObservable,
        ],
        "the report states exactly the closed ALL set, in its fixed order"
    );
}
