//! One record body in the durable Pilot ledger.

use serde::Serialize;

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::attempt_outcome::AttemptOutcome;
use crate::entity_owner_pilot::pilot_record_kind::PilotRecordKind;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;
use crate::manifest::schedule_seed::ScheduleSeed;

/// The body of one durable ledger line.
///
/// The three variants are exactly the three things the Parked Frontier requires the NDJSON to
/// contain — "block order, progressive rung evidence, and exactly one terminal
/// Complete/Failed/NotRun record for each of the ten predeclared attempts" — so the ledger's
/// contract and this enum are the same statement. A fourth thing cannot be written to the ledger,
/// and none of the three can be omitted without the sink's own accounting noticing.
///
/// Every variant that belongs to an attempt carries the full [`AttemptKey`], not a reference or an
/// index into the inventory. A ledger line is therefore independently interpretable: recovering what
/// a rung measured never requires replaying the inventory record first, which matters precisely in
/// the case the ledger exists for — a run that died partway.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum PilotRecord<'a> {
    /// The frozen inventory in execution order, with the seed it was derived from. Written once,
    /// before any attempt executes, so the recorded block order precedes the evidence it orders.
    Inventory {
        seed: ScheduleSeed,
        inventory: &'a AttemptInventory,
    },
    /// One completed rung of an in-flight attempt, appended as soon as that rung confirms.
    Rung {
        attempt: AttemptKey,
        evidence: &'a RungEvidence,
    },
    /// One predeclared attempt's single terminal disposition.
    Terminal {
        attempt: AttemptKey,
        outcome: &'a AttemptOutcome,
    },
}

impl PilotRecord<'_> {
    /// This record's kind, derived from its own variant so the tag written into the line's identity
    /// can never disagree with the body that follows it.
    pub(crate) fn kind(&self) -> PilotRecordKind {
        match self {
            Self::Inventory { .. } => PilotRecordKind::Inventory,
            Self::Rung { .. } => PilotRecordKind::Rung,
            Self::Terminal { .. } => PilotRecordKind::Terminal,
        }
    }
}
