//! A whole campaign ledger accounted against the frozen inventory it was written under.
//!
//! **Module topology is the enforcement mechanism here, not a comment.** The reconciler and the two
//! types only it may mint live together in the private, *childless* inline module [`sealed`]. Rust
//! makes a private field visible to its declaring module **and every descendant**, so declaring them
//! beside a `#[cfg(test)] mod tests` child — or any child added later — would let that module write
//! `SelectedCompleteAttempt { .. }` directly and assert a lowest-ordinal, non-superseded,
//! provenance-admitted selection that no accounting ever performed. That is exactly the guarantee the
//! ladder aggregate then rests on, so it must be the compiler's rather than a convention's.
//!
//! [`ReconciledCampaign`], [`AccountedAttemptRecord`], and [`SelectedCompleteAttempt`] are sealed
//! *together* because their minting is mutually dependent: the join and the fold are both methods of
//! the reconciler, and each output type is built by a struct literal inside it. All three are
//! re-exported, because all three appear in crate-visible signatures.
//!
//! [`ProvisioningDisposition`] deliberately stays outside `sealed`: an enum variant's payload cannot
//! be made private, so it carries no guarantee of its own and there is nothing for sole minting to
//! protect. Its guarantee lives one level up, in [`AccountedAttemptRecord`], which a hand-built
//! disposition has no way to enter.

use crate::view_read_set_campaign::attempt_provenance::AttemptProvenance;
use serde::Serialize;

/// Whether an attempt reached a published instance, carrying that instance's provenance when it did.
///
/// Freely constructible, harmlessly: an enum variant's payload cannot be made private, so this
/// carries no guarantee of its own. The guarantee lives one level up, at [`AccountedAttemptRecord`],
/// which has private fields sealed in a childless module and no constructor — so a hand-built
/// `ProvisioningDisposition` has nothing to go into.
///
/// A closed pair rather than an `Option<AttemptProvenance>` because the two cases mean different
/// things: `None` would say "no provenance recorded", which is indistinguishable from a lost line,
/// while [`Self::NeverPublished`] says the attempt provably never had one — a fact reconciliation
/// established from the terminal outcome's own typed lifecycle facts.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum ProvisioningDisposition {
    /// The attempt published, and this is the instance it measured against.
    Provisioned(AttemptProvenance),
    /// No instance was ever published under this identity, and none should have been: the attempt
    /// was never run, was refused by the prospective gate, or failed before publishing.
    NeverPublished,
}

mod sealed {
    use anyhow::Result;
    use serde::Serialize;

    use crate::view_read_set_campaign::attempt_inventory::AttemptInventory;
    use crate::view_read_set_campaign::attempt_key::AttemptKey;
    use crate::view_read_set_campaign::attempt_provenance::AttemptProvenance;
    use crate::view_read_set_campaign::campaign_ledger_line::CampaignLedgerLine;
    use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
    use crate::view_read_set_campaign::evidence_artifact::EvidenceArtifact;
    use crate::view_read_set_campaign::method_supersession::MethodSupersession;
    use crate::view_read_set_campaign::reconciled_campaign::ProvisioningDisposition;
    use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

    /// A campaign's whole ledger, accounted line by line against the frozen inventory it opens with.
    ///
    /// **Why this type exists.** Almost every claim analysis needs is a claim about *all* the lines
    /// at once — that no planned slot is missing, that no identity was written twice, that an attempt
    /// was gated and provisioned before it measured, that this is the lowest retry ordinal, that
    /// nothing supersedes it. None of those is derivable from one record or from an arbitrary slice,
    /// which is why the selection type below can only be minted here.
    ///
    /// **What it consumes, and why nothing less would do.** The complete ordered
    /// [`CampaignLedgerLine`] stream. Reconciliation over prefiltered vectors of terminal records
    /// could not see whether a launched attempt had a preflight clearance or a provisioned instance
    /// before it, nor whether the lines arrived in an order the protocol permits — so it could not
    /// establish the invariants it is here to establish, and a caller could pre-drop the very lines
    /// that would have falsified it. The inventory and the campaign pins likewise come *from* the
    /// stream rather than beside it, so a ledger cannot be accounted against a preregistration it was
    /// not written under.
    ///
    /// **Who can construct it.** Every field is private to this childless module and
    /// [`Self::reconciled`] is the only constructor, declared alongside them here. No other module —
    /// sibling, parent, or elsewhere in the crate — can assemble one, so a reconciled view cannot
    /// exist without the accounting having run.
    ///
    /// **What is validated and then deliberately not kept.** Two things, for the same reason: this
    /// type is the selection-capable decision surface, and a value it holds is a value a later rule
    /// can quietly condition on.
    ///
    /// - **Post-attempt environment readings.** The spec makes them supporting diagnostics that
    ///   "never invalidate evidence" and forbids any retry criterion from referencing a measured or
    ///   post-attempt outcome.
    /// - **Passing preflight clearances.** The retry rule needs only the *categorical*
    ///   [`PreflightRejected`](crate::view_read_set_campaign::attempt_outcome::AttemptOutcome::PreflightRejected)
    ///   disposition, which is a terminal outcome and stays. Retaining the passing gate *values*
    ///   would put load, RAM, swap, and PSI numbers within reach of selection as covariates —
    ///   evidence could then be picked by how quiet the host happened to be, which is not a criterion
    ///   this protocol has.
    ///
    /// Both are checked here for presence, cardinality, and position, and then stored nowhere: no
    /// field of this type holds one, and nothing reachable from a field holds one. A selection or
    /// retry rule written against this type cannot consult either, because the values are not in
    /// scope — not because a comment asked it not to. They remain on the ledger, which is where an
    /// auditor reads them; this type is a decision surface, not the archive.
    ///
    /// **What is validated and kept: provenance.** Selected evidence that has lost its execution
    /// provenance is not auditable, so the single [`CampaignProvenance`] and each applicable
    /// attempt's [`AttemptProvenance`] are retained, joined to their terminal records by
    /// [`AccountedAttemptRecord`]. They are an all-or-nothing admission gate — an attempt whose
    /// recorded runtime, module digest, or confirmed-read setting is off the campaign pins fails
    /// reconciliation outright — and never a per-attempt selection input, since a rule that preferred
    /// one runtime over another would be choosing evidence by its provenance.
    ///
    /// **Phase 1 boundary.** The accounting and the selection fold are explicit `todo!()` stubs. What
    /// is fixed here is the *signature*, which is what decides whether the checks are possible at
    /// all.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct ReconciledCampaign {
        inventory: AttemptInventory,
        provenance: CampaignProvenance,
        attempts: Vec<AccountedAttemptRecord>,
        supersessions: Vec<MethodSupersession>,
    }

    /// One terminal record joined to whatever provisioning provenance the ledger proved it must
    /// have.
    ///
    /// Named for *accounting*, not for execution: this covers every predeclared attempt, including
    /// the ones that never ran. A `NotRun` slot and a `PreflightRejected` refusal are both
    /// accounted-for records with no provisioned instance, and calling them "executed" would be false
    /// of exactly the cases the type exists to make visible.
    ///
    /// **Who can construct it.** Its fields are private to this childless module and it exposes no
    /// constructor, so the only code that can build one is the struct literal in
    /// [`ReconciledCampaign::reconciled`], sealed here alongside it. That is the point: the join is
    /// only sound once the whole ledger has been read, because whether an attempt *should* have a
    /// `Provisioned` line is a fact about its outcome and whether it *does* is a fact about the
    /// stream.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct AccountedAttemptRecord {
        terminal: TerminalAttemptRecord,
        provisioning: ProvisioningDisposition,
    }

    /// One logical slot's chosen attempt: complete, not superseded, and the lowest-ordinal such
    /// attempt.
    ///
    /// **Who can construct it.** Its fields are private to this childless module and it exposes no
    /// constructor, so the only code that can build one is the struct literal in
    /// [`ReconciledCampaign::selections`], sealed here alongside it. Confinement to a *childless*
    /// module is what makes that exact: a private field is visible to every descendant of its
    /// declaring module, so a type declared beside a `#[cfg(test)] mod tests` — or beside any child
    /// added later — would be directly constructible by it, and a `pub(super)` constructor would be
    /// visible to every sibling too. This mirrors
    /// [`EvidenceArtifact`](crate::view_read_set_campaign::evidence_artifact::EvidenceArtifact),
    /// whose variants are closed off the same way by an internal payload wrapper sealed alongside
    /// it.
    ///
    /// **Why that restriction is the whole point.** An earlier attempt at this type took a slice of
    /// terminal records and claimed to pick the lowest ordinal from it. That claim was not derivable
    /// from a slice: a caller could pass a subset that happened to omit ordinal 0, and the value
    /// would assert a minimum it had never seen. The guarantee is a fact about *every* record for the
    /// slot, so only a value holding every record of the whole ledger can establish it. That is what
    /// [`ReconciledCampaign`] is, and confining minting to it turns the claims below into
    /// consequences.
    ///
    /// **What holding one therefore establishes** — by construction of its minter, not by anything
    /// re-derivable from these two fields:
    ///
    /// - the slot's terminal records were the complete set, accounted against the frozen inventory;
    /// - among those whose outcome was
    ///   [`Complete`](crate::view_read_set_campaign::attempt_outcome::AttemptOutcome::Complete), this
    ///   is the one with the lowest
    ///   [`RetryOrdinal`](crate::view_read_set_campaign::retry_ordinal::RetryOrdinal);
    /// - no [`MethodSupersession`] in the campaign covers it;
    /// - its provisioning provenance agreed with the campaign pins, since a disagreement anywhere
    ///   fails reconciliation before any selection runs.
    ///
    /// **What it does not claim.** That the measurements are genuine — that remains a property of the
    /// driver's measurement path, as for every evidence type here. And it makes no claim about
    /// *other* slots: a ladder needs six of these plus its own check that they are six distinct rungs
    /// of one `(candidate, block, role, axis, version)`.
    ///
    /// It carries the [`EvidenceArtifact`] rather than the whole terminal record, so a holder cannot
    /// reach a non-complete outcome through it; and it deliberately omits
    /// [`MethodValidity`](crate::view_read_set_campaign::method_validity::MethodValidity), which
    /// records only what the campaign knew at write time and would read like a verdict the
    /// supersession fold has already replaced.
    ///
    /// **Why it carries the provenance, when reconciliation already gated on it.** A selected
    /// artifact travels: a ladder aggregate, and then a report, hold these rather than the campaign
    /// they came from. Without the provenance, answering "which runtime and module produced this
    /// number?" would mean looking the identity back up in a [`ReconciledCampaign`] the holder may no
    /// longer have. The value is a plain [`AttemptProvenance`] and is *not* a second admission gate —
    /// it was already admitted, and the selection fold that copies it here is forbidden to compare
    /// it, since a rule that preferred one runtime over another would be choosing evidence by its
    /// provenance. A [`Complete`](crate::view_read_set_campaign::attempt_outcome::AttemptOutcome::Complete)
    /// attempt is always provisioned, so there is no absent case to represent.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct SelectedCompleteAttempt {
        key: AttemptKey,
        artifact: EvidenceArtifact,
        provenance: AttemptProvenance,
    }

    impl ReconciledCampaign {
        /// Account a whole ledger against the inventory it opens with, failing loud on any missing,
        /// duplicated, unexplained, misplaced, or off-pin line.
        ///
        /// **The identity model this must implement, stated once here because it is subtle.** The
        /// frozen [`AttemptInventory`] predeclares *originals only*: it is built before execution,
        /// and a retry identity is derived later, from a terminal outcome that has not happened yet.
        /// So "every terminal identity must be predeclared" would reject every legitimate retry. What
        /// the ledger must instead show is one original per frozen logical slot, and a retry only
        /// where the original earned one.
        ///
        /// **Phase 1 boundary.** The checks land in Phase 2 and are frozen in the `todo!()` below.
        pub(crate) fn reconciled(lines: Vec<CampaignLedgerLine>) -> Result<Self> {
            let _ = lines;
            todo!(
                "Phase 2, over the ordered line stream:

                 Sequences — require the sequences to start at RecordSeq::zero and be contiguous and \
                 strictly ascending in stream order, so a hole or a reordering is a rejection rather \
                 than a gap a reader interpolates over.

                 Inventory — require exactly one CampaignRecord::Inventory, at the first line; take \
                 the frozen AttemptInventory and the CampaignProvenance from it, so the ledger is \
                 accounted against the preregistration it was actually written under.

                 Terminal identities — require exactly one Terminal per identity that appears. For \
                 each frozen logical slot of AttemptInventory::attempts, require exactly one \
                 terminal at RetryOrdinal::ORIGINAL. Permit at most one further terminal for that \
                 slot, at RetryOrdinal::RETRY, and only when the original's \
                 TerminalAttemptRecord::retry_eligibility is RetryEligibility::Eligible. Reject a \
                 retry whose original is absent or ineligible, a second retry, and any terminal \
                 identity matching no frozen slot at all.

                 Preflight clearances — total over the terminal outcome, because which attempts \
                 launched is exactly what the outcome says. Complete and Failed each require exactly \
                 one preceding PreflightCleared line for their identity: they launched. \
                 PreflightRejected requires none — its gate refused before launch and is already \
                 carried, as a FailedEnvironmentGate, inside the terminal record itself; a clearance \
                 for such an identity is a contradiction and is rejected. NotRun requires none, \
                 having never been gated. Reject a second clearance for one identity, and any \
                 clearance for an identity with no terminal record.

                 Provisioned provenance — total over the terminal outcome's own typed lifecycle \
                 facts, never inferred from a cause. Complete requires exactly one preceding \
                 Provisioned line: it measured, so it published. Failed requires exactly one if and \
                 only if its FailureStage::published is true, and none otherwise — which is why that \
                 stage is recorded, since FailureKind cannot distinguish a server-start Timeout from \
                 a reducer Timeout. PreflightRejected and NotRun require none. Reject a second \
                 provisioned line for one identity, and any provisioned line for an identity with no \
                 terminal record.

                 Provenance admission — run AttemptProvenance::agrees_with against the campaign \
                 provenance for every retained Provisioned line, and fail the whole reconciliation \
                 on any disagreement. This is all-or-nothing: a per-attempt exemption would let \
                 evidence be chosen by its runtime.

                 Post-attempt diagnostics — the spec requires one reading immediately after every \
                 measured attempt, so this is exactly-one, not at-most-one. Require exactly one \
                 PostAttemptEnvironment line, following that identity's terminal line, for Complete \
                 and for Failed whose FailureStage::measured_sample_boundary is \
                 MeasuredSampleBoundary::AfterFirst. Require none for Failed at BeforeFirst, for \
                 PreflightRejected, and for NotRun — none of them measured anything. Check it here \
                 and store none of it: it is deliberately absent from every field of this type.

                 Supersessions — require each Supersession's scope to reach at least one terminal \
                 identity present in this ledger, so an invalidation cannot silently name evidence \
                 this campaign never wrote."
            )
        }

        /// The frozen inventory this ledger was accounted against, taken from its own first line.
        pub(crate) fn inventory(&self) -> &AttemptInventory {
            &self.inventory
        }

        /// The campaign-constant pins every attempt here was admitted against.
        pub(crate) fn provenance(&self) -> &CampaignProvenance {
            &self.provenance
        }

        /// Every attempt of the campaign, originals and retries alike, each joined to its
        /// provisioning provenance. The ledger displays all of them; selection is a separate question
        /// answered by [`Self::selections`].
        pub(crate) fn attempts(&self) -> &[AccountedAttemptRecord] {
            &self.attempts
        }

        /// Every recorded method invalidation.
        pub(crate) fn supersessions(&self) -> &[MethodSupersession] {
            &self.supersessions
        }

        /// The selected attempt of every logical slot that has one.
        ///
        /// **Why the result may be shorter than the inventory, and why that is not an error.** A slot
        /// whose attempts all failed, or whose only complete attempt is superseded, contributes
        /// nothing — and the spec's answer to that is `Indeterminate(IncompleteEvidence)` at
        /// classification, not a rejected ledger. That is a different failure from a *missing terminal
        /// record*, which is a rejection and is caught in [`Self::reconciled`] before any selection
        /// runs.
        ///
        /// Infallible for the same reason: reconciliation already proved every identity here is
        /// accounted for, singly recorded, and provenance-admitted, so there is no unexplained record
        /// left for this fold to encounter.
        ///
        /// **Phase 1 boundary.** The fold lands in Phase 2, frozen in the `todo!()` below.
        pub(crate) fn selections(&self) -> Vec<SelectedCompleteAttempt> {
            todo!(
                "Phase 2: group this campaign's attempts by logical slot with \
                 AttemptKey::same_logical_slot; within each group keep only records whose outcome is \
                 AttemptOutcome::Complete and which no MethodSupersession covers; take the survivor \
                 with the lowest RetryOrdinal, and mint SelectedCompleteAttempt from that one \
                 record's key, its artifact, and the already-admitted AttemptProvenance its accounted \
                 record carries — a slot with no survivor yields nothing. Provenance is copied, never \
                 compared: it was an admission gate at reconciliation, not a preference here"
            )
        }
    }

    impl AccountedAttemptRecord {
        /// The attempt's identity and terminal disposition.
        pub(crate) fn terminal(&self) -> &TerminalAttemptRecord {
            &self.terminal
        }

        /// Whether the attempt reached a published instance, and that instance's provenance when it
        /// did.
        pub(crate) fn provisioning(&self) -> &ProvisioningDisposition {
            &self.provisioning
        }
    }

    impl SelectedCompleteAttempt {
        /// The full identity of the selected attempt, retry ordinal included — so a reader sees
        /// *which* attempt of the slot was taken, not merely that one was.
        pub(crate) fn key(&self) -> AttemptKey {
            self.key
        }

        /// The selected attempt's complete evidence.
        pub(crate) fn artifact(&self) -> &EvidenceArtifact {
            &self.artifact
        }

        /// The runtime and published instance this evidence was measured against, already admitted
        /// against the campaign pins — carried so the artifact stays auditable after it leaves the
        /// campaign it came from.
        pub(crate) fn provenance(&self) -> &AttemptProvenance {
            &self.provenance
        }
    }
}

// All three sealed types are intended crate-visible Phase-1 surface: outside code reconciles a
// ledger, walks `attempts()`, and holds selections. They are exposed as type aliases rather than
// `use` re-exports because only `SelectedCompleteAttempt` is *named* outside this file today, and an
// unused `use` is an `unused_imports` warning — which must never be silenced. An alias preserves the
// type, its methods, and its associated functions identically, and an unexercised one is ordinary
// dead code, already governed crate-wide by the skeleton's `#![allow(dead_code)]`. All three are
// aliased rather than mixing the two forms, so the file's surface reads uniformly.
pub(crate) type ReconciledCampaign = sealed::ReconciledCampaign;
pub(crate) type AccountedAttemptRecord = sealed::AccountedAttemptRecord;
pub(crate) type SelectedCompleteAttempt = sealed::SelectedCompleteAttempt;
