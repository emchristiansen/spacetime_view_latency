//! Which kind of record a durable Pilot ledger line carries.

use serde::Serialize;

/// The kind of a durable Pilot ledger record.
///
/// Deliberately *not* an extension of [`RecordKind`](crate::observation::record_kind::RecordKind),
/// whose two variants (`Manifest` and `Dose(DoseIndex)`) belong to the historical
/// Message/Chronicle campaign. The spec preserves that ontology rather than reinterpreting it, and a
/// Pilot rung is not a dose: it has no `DoseIndex`, no cumulative write ladder, and no manifest
/// lifecycle behind it. Adding variants there would have made every historical record's kind field
/// mean something slightly different retroactively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum PilotRecordKind {
    /// The frozen attempt inventory and its seed, written once before any attempt executes.
    Inventory,
    /// One rung's progressive evidence within an in-flight attempt.
    Rung,
    /// One predeclared attempt's single terminal disposition.
    Terminal,
}
