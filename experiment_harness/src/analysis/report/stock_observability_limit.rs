//! The closed set of stock-server observability limits the report must state explicitly.

use serde::Serialize;

/// The number of stock-server observability limits the report states explicitly (spec § Non-goals: the
/// two things a stock server cannot directly show without the separately-authorized escalation gate). It
/// fixes the limit array length, so *exactly these limits* is a property of the report type.
pub(crate) const STOCK_OBSERVABILITY_LIMIT_COUNT: usize = 2;

/// A limit of what the stock v2.6.1 server can be observed to prove directly, without the separately
/// authorized escalation gate's instrumentation of "read-set class and per-refresh materializer
/// operations". A closed typed enum, never a `String`, so the stated limits are a fixed, auditable set;
/// the experiment's stock evidence measures logical/client-visible change only, never these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum StockObservabilityLimit {
    /// A stock server cannot directly observe each view's read-set *class/granularity* — e.g. whether a
    /// refresh depended on a table-scoped versus a key-scoped read-set; that classification needs the
    /// patched-server read-set-class instrumentation behind the escalation gate.
    PerViewReadSetClassNotDirectlyObservable,
    /// A stock server cannot directly *count* each view's per-refresh materializer operations; delivered
    /// event counts measure net logical change, not hidden materializer work.
    PerViewMaterializerOperationCountsNotDirectlyObservable,
}

impl StockObservabilityLimit {
    /// The complete set of stock observability limits, in a fixed order — the exact set the report states.
    pub(crate) const ALL: [StockObservabilityLimit; STOCK_OBSERVABILITY_LIMIT_COUNT] = [
        StockObservabilityLimit::PerViewReadSetClassNotDirectlyObservable,
        StockObservabilityLimit::PerViewMaterializerOperationCountsNotDirectlyObservable,
    ];
}
