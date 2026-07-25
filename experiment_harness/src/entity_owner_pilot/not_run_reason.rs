//! Why a predeclared attempt was never executed.

use serde::Serialize;

/// Why a predeclared attempt produced no execution.
///
/// The spec requires the attempt inventory to be frozen *before* execution and one terminal record
/// appended for every planned attempt — "complete, failed with partial evidence and diagnostics, or
/// **not run with a reason**". Without this variant a stopped campaign would simply be missing
/// lines, and a missing line is indistinguishable from a line that was never planned. Recording the
/// reason is what keeps "the report generation fails on missing or duplicate planned identities"
/// checkable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum NotRunReason {
    /// The operator stopped the Pilot before the inventory was exhausted.
    Interrupted,
    /// The durable ledger became terminal (a persist failed), so no further attempt could be
    /// recorded honestly and execution stopped rather than measuring into a ledger that could not
    /// receive the result.
    LedgerTerminal,
}
