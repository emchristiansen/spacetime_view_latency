//! An appended record that evidence already on disk is no longer method-valid.

use serde::Serialize;

use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::superseded_scope::SupersededScope;
use crate::view_read_set_campaign::supersession_justification::SupersessionJustification;

/// One appended finding that a method was unsound, naming what it invalidates and why.
///
/// **Why this is a record and not a field.** The ledger is append-only, so a terminal record cannot
/// later *become* invalid: doing so would mean rewriting a line already on disk, which is exactly
/// what the spec forbids when it requires historical material be imported "without rewriting it".
/// [`MethodValidity`](super::method_validity::MethodValidity) therefore answers only "did the
/// campaign already know, when it wrote this line, that the method was unsound?" — and this record
/// carries everything learned afterwards. A selection rule that reads that field alone will select
/// superseded evidence; the rule must fold these records over the terminal outcomes, which is
/// [`ReconciledCampaign::selections`](super::reconciled_campaign::ReconciledCampaign::selections).
///
/// **What it is keyed by.** A [`SupersededScope`] and nothing else. The candidate version is inside
/// the scope — either within the named [`AttemptKey`] or as the version scope's own component — so
/// there is no second copy of it here to disagree with the attempts this record claims to describe,
/// the same reason [`CampaignProvenance`](super::campaign_provenance::CampaignProvenance) omits it.
///
/// **Who can construct it.** Any code in the crate, through [`Self::recorded`] — but only by
/// supplying a [`SupersessionJustification`], whose sole constructor rejects text that is empty
/// after trimming. So the structural guarantee is exactly the one the paragraph below claims: a
/// recorded invalidation always states a reason. Nothing stronger is enforceable here, because a
/// supersession is a *judgement* reached by a human or an analysis outside this campaign's
/// measurement path, and there is no observation for a constructor to check it against.
///
/// The scope carries the rest: [`SupersededScope::covers`] is exhaustive over it, so the selection
/// fold consumes these records totally instead of interpreting prose.
///
/// **What it does not claim.** That the justification is true, that its author was entitled to make
/// it, or that the named evidence is in fact unsound. It claims exactly that this invalidation was
/// recorded, against this extent, for this stated reason — which is what makes it auditable and
/// disputable rather than authoritative.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MethodSupersession {
    scope: SupersededScope,
    justification: SupersessionJustification,
}

impl MethodSupersession {
    /// Record one method invalidation against the extent it reaches.
    ///
    /// The justification is required rather than optional, and is a parsed type rather than a
    /// `String`: an invalidation with no stated reason removes evidence from consideration while
    /// leaving nothing to dispute, and `""` is a well-typed `String`.
    pub(crate) fn recorded(
        scope: SupersededScope,
        justification: SupersessionJustification,
    ) -> Self {
        Self {
            scope,
            justification,
        }
    }

    /// Whether this supersession reaches `attempt` — the primitive the selection fold is built from.
    pub(crate) fn covers(&self, attempt: AttemptKey) -> bool {
        self.scope.covers(attempt)
    }

    /// The extent this invalidation was recorded against.
    pub(crate) fn scope(&self) -> SupersededScope {
        self.scope
    }

    /// Why the method was found unsound, as recorded.
    pub(crate) fn justification(&self) -> &SupersessionJustification {
        &self.justification
    }
}
