//! Why a predeclared attempt was never executed.

use serde::Serialize;

/// Why a predeclared attempt produced no execution.
///
/// Without this, a stopped Pilot would simply be missing lines, and a missing line is
/// indistinguishable from one that was never planned — which is what makes "report generation fails
/// on missing or duplicate planned identities" checkable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum NotRunReason {
    /// The operator stopped the Pilot before the inventory was exhausted.
    Interrupted,
    /// The ledger became terminal, so no further result could be recorded honestly and execution
    /// stopped rather than measuring into a ledger that could not receive it.
    LedgerTerminal,
}
