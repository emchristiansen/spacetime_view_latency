//! One record body in the durable Pilot ledger.

use serde::Serialize;

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::attempt_outcome::AttemptOutcome;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;
use crate::entity_owner_pilot::campaign_provenance::PilotCampaignProvenance;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;
use crate::manifest::schedule_seed::ScheduleSeed;

/// The body of one durable ledger line.
///
/// The four variants are exactly the four things the ledger must contain — the campaign's frozen
/// order and pins, each attempt's provisioned instance, progressive rung evidence, and one terminal
/// disposition per predeclared attempt — so a fifth kind of line cannot be written.
///
/// Every attempt-bearing variant carries the full [`AttemptKey`] rather than an index into the
/// inventory, so lines join to each other by identity rather than by position. Provenance is
/// normalized onto `Inventory` and `Provisioned`, so a `Rung` or `Terminal` line is **not**
/// independently interpretable: reading it requires the ledger it belongs to.
///
/// The ledger's accounting and ordering invariants, which a reader must enforce rather than assume:
///
/// - exactly one `Inventory`, and it is the first line;
/// - for each attempt that was provisioned, exactly one `Provisioned`, before any `Rung` or
///   `Terminal` for that attempt — an attempt that failed before publish has none;
/// - `Rung` lines for one attempt are its ascending ladder prefix, each before that attempt's
///   `Terminal`;
/// - exactly one `Terminal` per predeclared attempt in the `Inventory`, and no `Terminal` for an
///   identity the `Inventory` does not predeclare.
///
/// A missing, duplicate, or out-of-order record of any of these kinds is a rejection, not a
/// repairable gap — which is what makes "report generation fails on missing or duplicate planned
/// identities" checkable.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum PilotRecord<'a> {
    /// The frozen inventory in execution order, the seed it came from, and the campaign-constant
    /// provenance every later line is interpreted under. Written once, before any attempt executes,
    /// so the recorded order and pins precede the evidence they govern.
    Inventory {
        seed: ScheduleSeed,
        inventory: &'a AttemptInventory,
        provenance: &'a PilotCampaignProvenance,
    },
    /// One attempt's actual runtime and freshly published instance, recorded after publish and
    /// before anything connects — so evidence can only ever be joined to the instance that produced
    /// it.
    Provisioned {
        attempt: AttemptKey,
        provenance: &'a AttemptProvenance,
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
            Self::Provisioned { .. } => "Provisioned",
            Self::Rung { .. } => "Rung",
            Self::Terminal { .. } => "Terminal",
        }
    }
}
