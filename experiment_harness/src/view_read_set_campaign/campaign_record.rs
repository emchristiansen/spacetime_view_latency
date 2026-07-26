//! One record body in the durable fresh-server campaign ledger.

use serde::Serialize;

use crate::view_read_set_campaign::attempt_inventory::AttemptInventory;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::attempt_provenance::AttemptProvenance;
use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;
use crate::view_read_set_campaign::method_supersession::MethodSupersession;
use crate::view_read_set_campaign::passed_environment_gate::PassedEnvironmentGate;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

/// The body of one durable ledger line.
///
/// Copies [`PilotRecord`](crate::entity_owner_pilot::pilot_record::PilotRecord)'s shape — one closed
/// enum of exactly the line kinds the ledger may contain, externally tagged by serde so the variant
/// name is in the line and no separate kind field can disagree with it — with two deliberate
/// differences, each forced by this protocol:
///
/// - **Owned, not borrowing.** The Pilot's record borrows because it is a write-only vocabulary. This
///   one is also the input to
///   [`ReconciledCampaign::reconciled`](super::reconciled_campaign::ReconciledCampaign), which holds
///   a whole campaign's records at once; a stream of borrows could not express that.
/// - **No progressive `Rung` line.** A rung was a step of the Pilot's cumulative walk, so evidence
///   accrued mid-attempt and needed durability before the terminal record. Here an attempt is one
///   scale point, and whatever it produced before failing is already retained inside its terminal
///   record's [`PartialEvidence`](super::partial_evidence::PartialEvidence).
///
/// **Why the two environment readings are different variants of different types.** The gate is
/// prospective and the post-attempt reading is retrospective, and the spec permits the first to
/// decide whether an attempt happens while forbidding the second from bearing on validity or retry
/// at all. One shared "environment sample" line would leave that distinction to a field a rule could
/// ignore. Two variants carrying two types means a decision surface that admits
/// [`PassedEnvironmentGate`] simply cannot be handed a [`Self::PostAttemptEnvironment`].
///
/// A refusal is *not* a line here: it is already the terminal record's
/// [`PreflightRejected`](super::attempt_outcome::AttemptOutcome::PreflightRejected) outcome, and
/// recording the same derived verdict twice would allow two copies to disagree. So the gate reaches
/// the ledger as a clearance on this line or as a refusal in the terminal record, never both.
///
/// Every attempt-bearing variant carries the full [`AttemptKey`], so lines join to each other by
/// identity rather than by position — except [`Self::Terminal`], which carries a
/// [`TerminalAttemptRecord`] that already binds a key to its outcome and would duplicate the key if
/// it also had a field for one. Provenance is normalized onto [`Self::Inventory`] and
/// [`Self::Provisioned`], so a `Terminal` line is **not** independently interpretable: reading it
/// requires the ledger it belongs to.
///
/// The accounting and ordering invariants a reader must enforce rather than assume are stated once,
/// where they are checked — see [`ReconciledCampaign::reconciled`](super::reconciled_campaign::ReconciledCampaign).
#[derive(Debug, Clone, Serialize)]
pub(crate) enum CampaignRecord {
    /// The frozen inventory in execution order and the campaign-constant provenance every later line
    /// is interpreted under. Written once, before any attempt executes, so the recorded order and
    /// pins precede the evidence they govern.
    ///
    /// No seed field: unlike the Pilot, whose block order came from a CLI argument, this campaign's
    /// order derives from the frozen `CAMPAIGN_SEED`, which is recorded literally inside
    /// [`CampaignProvenance`]'s parameters. A second copy here could disagree with it.
    Inventory {
        inventory: AttemptInventory,
        provenance: CampaignProvenance,
    },
    /// The prospective gate readings that cleared one attempt to launch, appended before it launches.
    /// Its absence before a measured terminal record means an attempt ran ungated.
    PreflightCleared {
        attempt: AttemptKey,
        gate: PassedEnvironmentGate,
    },
    /// One attempt's actual runtime and freshly published instance, recorded after publish and
    /// before anything connects — so evidence can only ever be joined to the instance that produced
    /// it.
    Provisioned {
        attempt: AttemptKey,
        provenance: AttemptProvenance,
    },
    /// One attempt's single terminal disposition, bound to the identity it describes.
    Terminal { record: TerminalAttemptRecord },
    /// The host reading taken immediately after a measured attempt. Supporting diagnostics only: it
    /// never invalidates evidence and no retry or selection criterion may reference it, which is
    /// enforced by keeping it out of the reconciled decision surface entirely.
    PostAttemptEnvironment {
        attempt: AttemptKey,
        sample: EnvironmentSample,
    },
    /// An appended finding that evidence already on disk is no longer method-valid. Append-only: it
    /// supersedes earlier lines without rewriting them.
    Supersession { supersession: MethodSupersession },
}

impl CampaignRecord {
    /// This variant's name, for diagnostics only.
    ///
    /// A persist failure must name the record it lost, but on a *serialization* failure the body
    /// cannot be rendered — hence a plain discriminant name rather than reading the serialized line.
    /// This is not written into the record: serde's external tagging already emits the variant name
    /// in the line itself.
    pub(crate) fn variant_name(&self) -> &'static str {
        match self {
            Self::Inventory { .. } => "Inventory",
            Self::PreflightCleared { .. } => "PreflightCleared",
            Self::Provisioned { .. } => "Provisioned",
            Self::Terminal { .. } => "Terminal",
            Self::PostAttemptEnvironment { .. } => "PostAttemptEnvironment",
            Self::Supersession { .. } => "Supersession",
        }
    }
}
