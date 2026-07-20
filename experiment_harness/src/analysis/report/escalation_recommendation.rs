//! The closed stock-to-patch escalation recommendation.

use serde::Serialize;

/// The typed outcome of the stock-to-patch escalation gate (spec § "Stock-to-patched-server escalation
/// gate"): whether the stock evidence answers the core question or a new scope decision is warranted. A
/// closed enum, never a `String`, and it never patches or authorizes patching — it only records the
/// recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum EscalationRecommendation {
    /// The stock outcome answers the core question (spec: `{A–E}` Increasing and `{F, F′}` Flat-equivalent
    /// under unrelated growth), so no patch is needed and the experiment remains on the stock server.
    RemainStock,
    /// The stock outcome meets an escalation condition, so a separately-authorized new scope decision —
    /// possibly a v2.6.1-pinned patched server instrumenting read-set class and materializer operations —
    /// is warranted. Recording this never authorizes the patch.
    EscalateToPatch,
}
