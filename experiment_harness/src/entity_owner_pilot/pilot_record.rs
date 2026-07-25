//! One record body in the durable Pilot ledger.

use serde::Serialize;

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::attempt_outcome::AttemptOutcome;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;
use crate::manifest::schedule_seed::ScheduleSeed;

/// The body of one durable ledger line.
///
/// The three variants are exactly the three things the ledger must contain — block order,
/// progressive rung evidence, and one terminal disposition per predeclared attempt — so a fourth
/// kind of line cannot be written.
///
/// Every attempt-bearing variant carries the full [`AttemptKey`] rather than an index into the
/// inventory, so a line is independently interpretable without replaying earlier lines — which
/// matters precisely in the case the ledger exists for, a run that died partway.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum PilotRecord<'a> {
    /// The frozen inventory in execution order with the seed it came from. Written once, before any
    /// attempt executes, so the recorded order precedes the evidence it orders.
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
    /// This variant's name, for diagnostics only.
    ///
    /// A persist failure must name the record it lost, but on a *serialization* failure the body
    /// cannot be rendered — hence a plain discriminant name rather than reading the serialized line.
    /// This is not written into the record: serde's external tagging already emits the variant name
    /// in the line itself.
    pub(crate) fn variant_name(&self) -> &'static str {
        match self {
            Self::Inventory { .. } => "Inventory",
            Self::Rung { .. } => "Rung",
            Self::Terminal { .. } => "Terminal",
        }
    }
}
