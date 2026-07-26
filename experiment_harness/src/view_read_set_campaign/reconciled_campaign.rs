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
    use anyhow::{bail, ensure, Context, Result};
    use serde::Serialize;

    use crate::observation::record_seq::RecordSeq;
    use crate::view_read_set_campaign::attempt_inventory::AttemptInventory;
    use crate::view_read_set_campaign::attempt_key::AttemptKey;
    use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
    use crate::view_read_set_campaign::attempt_provenance::AttemptProvenance;
    use crate::view_read_set_campaign::campaign_ledger_line::CampaignLedgerLine;
    use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
    use crate::view_read_set_campaign::campaign_record::CampaignRecord;
    use crate::view_read_set_campaign::evidence_artifact::EvidenceArtifact;
    use crate::view_read_set_campaign::measured_sample_boundary::MeasuredSampleBoundary;
    use crate::view_read_set_campaign::method_supersession::MethodSupersession;
    use crate::view_read_set_campaign::method_validity::MethodValidity;
    use crate::view_read_set_campaign::reconciled_campaign::ProvisioningDisposition;
    use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
    use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;
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
    /// **Both halves now exist.** [`Self::reconciled`] establishes the whole-ledger facts;
    /// [`Self::selections`] folds the supersessions over them. The second is only sound because the
    /// first ran, which is why they are methods of one type rather than two passes a caller
    /// sequences.
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
        /// **The passes, in the order a failure is most usefully reported.** Each is a free function
        /// below, and each states the rule it enforces where it enforces it:
        /// [`contiguous_from_zero`], [`opening_inventory`], [`bucketed_by_identity`],
        /// [`account_identities`], [`AttemptLines::account_auxiliary_lines`],
        /// [`AttemptLines::admit_provenance`], [`reaching_supersessions`], and finally
        /// [`accounted_in_terminal_order`], which is infallible because every preceding pass has
        /// already established what it joins.
        ///
        /// The ordering is deliberate where two rules overlap: a ledger missing a predeclared
        /// original *and* carrying a retry of it violates both the inventory rule and the retry
        /// rule, and the inventory rule names the missing slot, which is the more actionable of the
        /// two reports.
        pub(crate) fn reconciled(lines: Vec<CampaignLedgerLine>) -> Result<Self> {
            contiguous_from_zero(&lines)?;
            let (inventory, provenance) = opening_inventory(&lines)?;

            let identities = bucketed_by_identity(&lines)?;
            account_identities(&identities, &inventory)?;
            for identity in &identities {
                identity.account_auxiliary_lines()?;
                identity.admit_provenance(&provenance)?;
            }
            let supersessions = reaching_supersessions(&lines, &identities)?;
            let attempts = accounted_in_terminal_order(&lines, &identities);

            Ok(Self {
                inventory,
                provenance,
                attempts,
                supersessions,
            })
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
        /// **The slots come from the frozen inventory, not from the records.** Reconciliation proved
        /// the two coincide — every predeclared slot has its original, and every recorded identity
        /// addresses a predeclared slot — but "per logical slot" names the *preregistration*, and
        /// reading the slots from it keeps the result's membership a property of the frozen plan
        /// rather than of whatever the ledger happened to contain. The output order is therefore the
        /// inventory's; it is deliberately not ledger order, and nothing downstream may depend on
        /// either, since analysis reconstructs ladders by scale-point identity.
        ///
        /// **Provenance is copied, never compared.** It was an admission gate at reconciliation; a
        /// rule here that preferred one runtime over another would be choosing evidence by its
        /// provenance.
        pub(crate) fn selections(&self) -> Vec<SelectedCompleteAttempt> {
            let mut selections = Vec::new();
            for slot in self.inventory.attempts() {
                let Some(selected) = self.lowest_ordinal_survivor(*slot) else {
                    continue;
                };
                selections.push(SelectedCompleteAttempt {
                    key: selected.key,
                    artifact: selected.artifact.clone(),
                    provenance: selected.provenance.clone(),
                });
            }
            selections
        }

        /// The one attempt a logical slot contributes: complete, method-valid as written, reached by
        /// no supersession, and the lowest such retry ordinal.
        ///
        /// **Exclusion strictly precedes the minimum, because that is the general rule the spec
        /// states** — select the lowest ordinal *among the survivors*, not "the lowest ordinal, kept
        /// only if it survives". The two differ exactly when a slot holds more than one complete
        /// candidate and the lower-ordinal one is superseded: this order then selects the surviving
        /// higher ordinal, while the reversed order would discard the slot entirely.
        ///
        /// **That difference is vacuous under today's retry policy, and is written this way anyway.**
        /// Reconciliation admits a retry only where the original's
        /// [`retry_eligibility`](crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord::retry_eligibility)
        /// is [`RetryEligibility::Eligible`], which no complete outcome ever is, so at most one
        /// complete candidate per slot is currently representable and both orders agree on every
        /// reachable ledger. The ordering is therefore direct-inspection-only, exactly as the test
        /// module records. It is the spec's rule rather than an encoding of the present policy
        /// because the policy is the part that can change: were multiple complete candidates ever
        /// admitted, this order would already be the correct one, with no compile error to prompt a
        /// revisit.
        ///
        /// **Supersessions are matched against the full key, retry ordinal included.** That is what
        /// [`SupersededScope`](crate::view_read_set_campaign::superseded_scope::SupersededScope)
        /// means: an attempt-scoped invalidation of an original says nothing about its retry, while
        /// a candidate-version scope reaches both. Excluding by logical slot would over-delete the
        /// first case.
        ///
        /// The minimum needs no tie rule. Reconciliation admits one terminal record per identity,
        /// and an identity is its slot plus its ordinal, so two candidates of one slot cannot share
        /// an ordinal.
        fn lowest_ordinal_survivor(&self, slot: AttemptKey) -> Option<SelectedEvidence<'_>> {
            self.attempts
                .iter()
                .filter(|record| record.terminal().key().same_logical_slot(slot))
                .filter_map(selectable)
                .filter(|selected| !self.superseded(selected.key))
                .min_by_key(|selected| selected.key.retry())
        }

        /// Whether any invalidation this ledger recorded reaches exactly this attempt identity.
        fn superseded(&self, attempt: AttemptKey) -> bool {
            self.supersessions
                .iter()
                .any(|supersession| supersession.covers(attempt))
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

    /// Everything one selectable record contributes, borrowed from the accounted record holding it.
    ///
    /// Extracted before the ordinal comparison rather than after it, so the bridge in
    /// [`selectable`] is crossed once per candidate and never again on the winner — the selection is
    /// then a copy of facts already in hand.
    struct SelectedEvidence<'a> {
        key: AttemptKey,
        artifact: &'a EvidenceArtifact,
        provenance: &'a AttemptProvenance,
    }

    /// What an accounted record contributes to its slot's selection, or `None` when its terminal
    /// outcome makes it no candidate at all.
    ///
    /// **The validity state is total-matched together with the outcome, deliberately.**
    /// [`MethodValidity`] has one variant today, so naming it changes nothing that runs. But it
    /// answers "did the campaign already know, as it wrote this line, that the method was unsound?",
    /// and a future state meaning *yes* would otherwise be selected in silence. Written this way,
    /// adding one fails to compile here until it states its own selection disposition. This does not
    /// replace the supersession fold, which carries everything learned *after* the line was written;
    /// the two answer different questions and both must hold.
    ///
    /// **The one `.expect` bridges a guarantee the accounting established and this pair cannot
    /// carry.** [`required_provisioned`] returns 1 for a complete outcome and
    /// [`AttemptLines::account_family`] refused any attempt whose provisioned-line count differed, so
    /// a complete record reaching here always has its admitted provenance —
    /// [`AccountedAttemptRecord`] simply stores the outcome and the disposition side by side rather
    /// than in one variant, so the compiler cannot see it. The alternatives were worse: silently
    /// dropping admitted evidence, or a fallible signature contradicting this fold's frozen
    /// infallibility. A variant coupling complete evidence to its provenance would make the bridge
    /// unnecessary, but that is a rewrite of the sealed accounted-record boundary rather than part of
    /// this fold. Same shape and same justification as [`AttemptLines::terminal`].
    fn selectable(record: &AccountedAttemptRecord) -> Option<SelectedEvidence<'_>> {
        let artifact = match record.terminal().outcome() {
            AttemptOutcome::Complete {
                artifact,
                validity: MethodValidity::Valid,
            } => artifact,
            AttemptOutcome::PreflightRejected { .. }
            | AttemptOutcome::Failed { .. }
            | AttemptOutcome::NotRun { .. } => return None,
        };
        let provenance = match record.provisioning() {
            ProvisioningDisposition::Provisioned(provenance) => Some(provenance),
            ProvisioningDisposition::NeverPublished => None,
        }
        .expect(
            "reconciliation requires exactly one provisioned line for every complete attempt, so a \
             complete accounted record always carries its admitted provenance",
        );
        Some(SelectedEvidence {
            key: record.terminal().key(),
            artifact,
            provenance,
        })
    }

    /// Every line the ledger wrote under one attempt identity, gathered so each frozen per-attempt
    /// rule is checked against the terminal outcome that explains it.
    ///
    /// Borrows rather than owns: reconciliation may reject, and cloning a whole ledger's records to
    /// discover that would be work done for nothing. Only [`accounted_in_terminal_order`] clones,
    /// and only after every pass has accepted.
    struct AttemptLines<'a> {
        key: AttemptKey,
        terminal_line: Option<(RecordSeq, &'a TerminalAttemptRecord)>,
        clearances: Vec<RecordSeq>,
        provisioned: Vec<(RecordSeq, &'a AttemptProvenance)>,
        post_attempt: Vec<RecordSeq>,
    }

    /// Where an auxiliary line must sit relative to the terminal line of its own attempt.
    ///
    /// Only *relative to its own terminal*: retry adjacency and cross-identity execution ordering
    /// are properties of the driver's schedule, not of the ledger, and reconciliation does not
    /// re-derive them.
    /// **The two are not the same strength, because their frozen clauses are not.** A clearance and
    /// a provisioned line are required to *precede* their terminal line and nothing more: the
    /// protocol writes each as soon as the fact it records becomes true — the gate passed, the
    /// module published — and an arbitrary number of other attempts' lines may legitimately fall
    /// between one of them and the terminal record it belongs to. The post-attempt reading is
    /// different: the spec requires it *immediately after* the measured attempt, and a reading taken
    /// once other work has intervened is a reading of a different host state than the one the
    /// measurement finished in.
    #[derive(Clone, Copy)]
    enum RequiredPosition {
        /// Written at any earlier sequence than the terminal line — the clearance that let the
        /// attempt launch, and the provenance of the instance it published.
        BeforeTerminal,
        /// Written at exactly the sequence after the terminal line, with nothing in between.
        ImmediatelyAfterTerminal,
    }

    impl RequiredPosition {
        /// Whether `seq` sits where this position requires, relative to `terminal_seq`.
        fn admits(self, seq: RecordSeq, terminal_seq: RecordSeq) -> bool {
            match self {
                Self::BeforeTerminal => seq < terminal_seq,
                Self::ImmediatelyAfterTerminal => seq == terminal_seq.next(),
            }
        }

        /// How a refusal spells this requirement.
        fn requirement(self) -> &'static str {
            match self {
                Self::BeforeTerminal => "at some sequence before",
                Self::ImmediatelyAfterTerminal => "at the sequence immediately after",
            }
        }
    }

    impl<'a> AttemptLines<'a> {
        /// A bucket for `key` with no lines in it yet.
        fn empty(key: AttemptKey) -> Self {
            Self {
                key,
                terminal_line: None,
                clearances: Vec::new(),
                provisioned: Vec::new(),
                post_attempt: Vec::new(),
            }
        }

        /// This identity's single terminal line.
        ///
        /// Infallible past [`account_identities`], which refuses any identity the ledger wrote a
        /// line for without also writing its terminal record.
        fn terminal(&self) -> (RecordSeq, &'a TerminalAttemptRecord) {
            self.terminal_line
                .expect("every recorded attempt identity was proven to have one terminal record")
        }

        /// Require this identity's auxiliary lines to be exactly the ones its terminal outcome
        /// calls for, each where the protocol writes it relative to that terminal line.
        fn account_auxiliary_lines(&self) -> Result<()> {
            let (terminal_seq, terminal) = self.terminal();
            let outcome = terminal.outcome();

            let provisioned: Vec<RecordSeq> =
                self.provisioned.iter().map(|(seq, _)| *seq).collect();

            self.account_family(
                "preflight clearance",
                &self.clearances,
                required_clearances(outcome),
                RequiredPosition::BeforeTerminal,
                terminal_seq,
            )?;
            self.account_family(
                "provisioned provenance",
                &provisioned,
                required_provisioned(outcome),
                RequiredPosition::BeforeTerminal,
                terminal_seq,
            )?;
            self.account_family(
                "post-attempt environment",
                &self.post_attempt,
                required_post_attempt(outcome),
                RequiredPosition::ImmediatelyAfterTerminal,
                terminal_seq,
            )
        }

        /// Require one auxiliary line family to have exactly `required` members, each at a sequence
        /// [`RequiredPosition`] admits.
        ///
        /// Cardinality first, then position: a duplicate and a misplaced line are different
        /// mistakes, and reporting "there are two of these" before "this one is in the wrong place"
        /// is the order that names the actual defect.
        fn account_family(
            &self,
            family: &str,
            seqs: &[RecordSeq],
            required: usize,
            position: RequiredPosition,
            terminal_seq: RecordSeq,
        ) -> Result<()> {
            ensure!(
                seqs.len() == required,
                "attempt {} has {} {family} line(s), but its terminal outcome requires exactly \
                 {required}",
                self.key.canonical_tag(),
                seqs.len(),
            );
            for seq in seqs {
                ensure!(
                    position.admits(*seq, terminal_seq),
                    "attempt {}'s {family} line is at sequence {}, but must be written {} its \
                     terminal line at sequence {}",
                    self.key.canonical_tag(),
                    seq.get(),
                    position.requirement(),
                    terminal_seq.get(),
                );
            }
            Ok(())
        }

        /// Check every provisioned instance this identity recorded against the campaign's pins.
        ///
        /// All-or-nothing by being a `?` inside the whole-ledger constructor: one disagreeing
        /// attempt fails the reconciliation rather than being dropped from it. A per-attempt
        /// exemption would let evidence be chosen by the runtime it happened to be measured on.
        fn admit_provenance(&self, campaign: &CampaignProvenance) -> Result<()> {
            for (seq, provenance) in &self.provisioned {
                provenance.agrees_with(campaign).with_context(|| {
                    format!(
                        "attempt {}'s provisioned instance, recorded at sequence {}, disagrees with \
                         this campaign's pins",
                        self.key.canonical_tag(),
                        seq.get(),
                    )
                })?;
            }
            Ok(())
        }

        /// Whether this attempt reached a published instance, and the provenance of the one it
        /// reached.
        ///
        /// Reading the first provisioned line is exact rather than approximate: cardinality was
        /// already checked *total over the terminal outcome*, so a line is present here precisely
        /// when the outcome required one.
        fn provisioning(&self) -> ProvisioningDisposition {
            match self.provisioned.first() {
                Some((_, provenance)) => {
                    ProvisioningDisposition::Provisioned((*provenance).clone())
                }
                None => ProvisioningDisposition::NeverPublished,
            }
        }
    }

    /// How many preflight clearances an attempt with this outcome must have.
    ///
    /// Total over the outcome, because which attempts launched is exactly what the outcome says.
    /// [`AttemptOutcome::Complete`] and [`AttemptOutcome::Failed`] launched.
    /// [`AttemptOutcome::PreflightRejected`] did not — its gate refused before launch and is
    /// already carried, as a
    /// [`FailedEnvironmentGate`](crate::view_read_set_campaign::failed_environment_gate::FailedEnvironmentGate),
    /// inside the terminal record itself, so a clearance for that identity is a contradiction.
    /// [`AttemptOutcome::NotRun`] was never gated.
    fn required_clearances(outcome: &AttemptOutcome) -> usize {
        match outcome {
            AttemptOutcome::Complete { .. } | AttemptOutcome::Failed { .. } => 1,
            AttemptOutcome::PreflightRejected { .. } | AttemptOutcome::NotRun { .. } => 0,
        }
    }

    /// How many provisioned-provenance lines an attempt with this outcome must have.
    ///
    /// Total over the outcome's own *typed lifecycle facts*, never inferred from a cause: a
    /// complete attempt measured, so it published, and a failed one published exactly when
    /// [`FailureStage::published`](crate::view_read_set_campaign::failure_stage::FailureStage::published)
    /// says so. That stage is recorded precisely because
    /// [`FailureKind`](crate::view_read_set_campaign::failure_kind::FailureKind) cannot distinguish
    /// a server-start timeout from a reducer timeout.
    fn required_provisioned(outcome: &AttemptOutcome) -> usize {
        match outcome {
            AttemptOutcome::Complete { .. } => 1,
            AttemptOutcome::Failed { stage, .. } => usize::from(stage.published()),
            AttemptOutcome::PreflightRejected { .. } | AttemptOutcome::NotRun { .. } => 0,
        }
    }

    /// How many post-attempt environment readings an attempt with this outcome must have.
    ///
    /// Exactly-one rather than at-most-one for a measured attempt: the spec requires a reading
    /// immediately after every measured attempt, so a missing one is a gap in the diagnostics
    /// record. That "immediately" is carried by
    /// [`RequiredPosition::ImmediatelyAfterTerminal`] rather than by this count, and the two
    /// together are the whole clause. Nothing measured means none — and none of these values is
    /// stored anywhere on the reconciled view, by design.
    fn required_post_attempt(outcome: &AttemptOutcome) -> usize {
        match outcome {
            AttemptOutcome::Complete { .. } => 1,
            AttemptOutcome::Failed { stage, .. } => match stage.measured_sample_boundary() {
                MeasuredSampleBoundary::AfterFirst => 1,
                MeasuredSampleBoundary::BeforeFirst => 0,
            },
            AttemptOutcome::PreflightRejected { .. } | AttemptOutcome::NotRun { .. } => 0,
        }
    }

    /// Require the ledger's sequences to start at zero and step by one in stream order.
    ///
    /// One check covers a nonzero start, a hole, a duplicate, and a reordering, because all four
    /// are the same defect seen from different sides: the position a line claims is not the
    /// position it occupies. A hole must be a rejection rather than a gap a reader interpolates
    /// over, since the missing line could be the very clearance or terminal record an accounting
    /// rule turns on.
    fn contiguous_from_zero(lines: &[CampaignLedgerLine]) -> Result<()> {
        let mut expected = RecordSeq::zero();
        for line in lines {
            ensure!(
                line.seq() == expected,
                "a campaign ledger's sequences must start at {} and be contiguous in stream order; \
                 found {} where {} was expected",
                RecordSeq::zero().get(),
                line.seq().get(),
                expected.get(),
            );
            expected = expected.next();
        }
        Ok(())
    }

    /// Take the frozen preregistration from the ledger's own first line, requiring exactly one.
    ///
    /// *From* the stream rather than beside it: a ledger accounted against an inventory it was not
    /// written under would check the wrong slots, and pins supplied by the caller would let the
    /// admission gate be chosen after the fact.
    fn opening_inventory(
        lines: &[CampaignLedgerLine],
    ) -> Result<(AttemptInventory, CampaignProvenance)> {
        let first = lines
            .first()
            .context("a campaign ledger must open with its frozen inventory, but has no lines")?;
        let CampaignRecord::Inventory {
            inventory,
            provenance,
        } = first.body()
        else {
            bail!(
                "a campaign ledger must open with its frozen inventory; its first line is a {} \
                 record",
                first.body().variant_name(),
            );
        };
        for line in lines.iter().skip(1) {
            ensure!(
                !matches!(line.body(), CampaignRecord::Inventory { .. }),
                "a campaign ledger records exactly one inventory, at its first line; a second one \
                 is at sequence {}",
                line.seq().get(),
            );
        }
        Ok((inventory.clone(), provenance.clone()))
    }

    /// Gather every line under the identity it names, rejecting a second terminal record for one
    /// identity.
    ///
    /// Duplicate terminals are caught here rather than in a later pass because this is where the
    /// two would collide: a bucket holds one terminal line, so the second has nowhere to go and is
    /// refused at the moment it is seen, with its own sequence to name.
    fn bucketed_by_identity(lines: &[CampaignLedgerLine]) -> Result<Vec<AttemptLines<'_>>> {
        let mut identities: Vec<AttemptLines> = Vec::new();
        for line in lines {
            let seq = line.seq();
            match line.body() {
                // Neither names an attempt identity: the inventory is campaign-wide, and a
                // supersession's reach is checked against the identities rather than filed under
                // one.
                CampaignRecord::Inventory { .. } | CampaignRecord::Supersession { .. } => {}
                CampaignRecord::PreflightCleared { attempt, .. } => {
                    bucket(&mut identities, *attempt).clearances.push(seq);
                }
                CampaignRecord::Provisioned {
                    attempt,
                    provenance,
                } => {
                    bucket(&mut identities, *attempt)
                        .provisioned
                        .push((seq, provenance));
                }
                CampaignRecord::PostAttemptEnvironment { attempt, .. } => {
                    bucket(&mut identities, *attempt).post_attempt.push(seq);
                }
                CampaignRecord::Terminal { record } => {
                    let identity = bucket(&mut identities, record.key());
                    if let Some((first_seq, _)) = identity.terminal_line {
                        bail!(
                            "attempt {} has more than one terminal record: sequences {} and {}",
                            record.key().canonical_tag(),
                            first_seq.get(),
                            seq.get(),
                        );
                    }
                    identity.terminal_line = Some((seq, record));
                }
            }
        }
        Ok(identities)
    }

    /// The bucket for `key`, created empty the first time that identity is seen.
    ///
    /// Linear search on [`AttemptKey`] equality rather than a hash lookup, deliberately: hashing
    /// would mean deriving [`std::hash::Hash`] on the frozen identity type for the benefit of one
    /// reconciliation pass. A campaign predeclares sixty originals and at most one representable
    /// retry each, so the bounded quadratic comparison is negligible, and the identity type stays
    /// exactly as small as its own contract requires.
    fn bucket<'a, 'b>(
        identities: &'b mut Vec<AttemptLines<'a>>,
        key: AttemptKey,
    ) -> &'b mut AttemptLines<'a> {
        let position = match identities.iter().position(|identity| identity.key == key) {
            Some(position) => position,
            None => {
                identities.push(AttemptLines::empty(key));
                identities
                    .len()
                    .checked_sub(1)
                    .expect("a vector just pushed to is nonempty")
            }
        };
        identities
            .get_mut(position)
            .expect("a position taken from this vector indexes it")
    }

    /// Account every recorded identity against the frozen inventory, and every retry against the
    /// original that had to earn it.
    ///
    /// **The identity model, which is why this is two loops rather than a set comparison.** The
    /// inventory predeclares originals only, so "every terminal identity must be predeclared" would
    /// reject every legitimate retry, and "every predeclared identity must appear" says nothing
    /// about the retries. The first loop requires each frozen slot's original; the second requires
    /// each recorded identity to be either that original or a permitted retry of it.
    ///
    /// **A second retry is unrepresentable, so it is not checked.**
    /// [`RetryOrdinal`](crate::view_read_set_campaign::retry_ordinal::RetryOrdinal) has exactly two
    /// values, buckets are keyed by whole-identity equality, and one bucket holds one terminal — so
    /// a slot cannot carry a third terminal record at all. The cap is the type's, not this
    /// function's.
    fn account_identities(
        identities: &[AttemptLines<'_>],
        inventory: &AttemptInventory,
    ) -> Result<()> {
        for slot in inventory.attempts() {
            ensure!(
                identities
                    .iter()
                    .any(|identity| identity.key == *slot && identity.terminal_line.is_some()),
                "the frozen inventory predeclares attempt {}, for which this ledger has no terminal \
                 record",
                slot.canonical_tag(),
            );
        }

        for identity in identities {
            ensure!(
                identity.terminal_line.is_some(),
                "the ledger writes preflight, provisioning or post-attempt lines for attempt {}, \
                 which has no terminal record of its own",
                identity.key.canonical_tag(),
            );

            // Currently unreachable, and kept as a runtime rejection rather than an `expect`
            // precisely because it is unreachable only by accident of today's vocabulary: every
            // component of an `AttemptKey` is closed to the values this one campaign uses, so the
            // frozen inventory happens to be the complete cross product of every representable
            // logical slot. A later candidate, axis, block, or candidate version makes foreign
            // identities representable without changing a line here, and an `expect` would then be
            // a false claim rather than a compile error.
            let slot = inventory
                .attempts()
                .iter()
                .find(|frozen| frozen.same_logical_slot(identity.key))
                .with_context(|| {
                    format!(
                        "the ledger records attempt {}, which addresses no logical slot the frozen \
                         inventory predeclares",
                        identity.key.canonical_tag(),
                    )
                })?;

            // An identity at `ORIGINAL` sharing a logical slot with a frozen entry *is* that entry:
            // the slot comparison covers every component but the retry ordinal, and every frozen
            // entry is an original. So only a retry has anything left to establish.
            if identity.key.retry() != RetryOrdinal::ORIGINAL {
                let (_, original) = identities
                    .iter()
                    .find(|other| other.key == *slot)
                    .and_then(|other| other.terminal_line)
                    .with_context(|| {
                        format!(
                            "attempt {} retries a logical slot whose original has no terminal \
                             record in this ledger",
                            identity.key.canonical_tag(),
                        )
                    })?;
                ensure!(
                    original.retry_eligibility() == RetryEligibility::Eligible,
                    "attempt {} retries a logical slot whose original is {:?} for a retry",
                    identity.key.canonical_tag(),
                    original.retry_eligibility(),
                );
            }
        }
        Ok(())
    }

    /// Collect the ledger's supersessions, requiring each to reach an attempt this ledger records.
    ///
    /// Reach *anywhere* in the ledger, not merely earlier: a
    /// [`CandidateVersion`](crate::view_read_set_campaign::superseded_scope::SupersededScope::CandidateVersion)
    /// scope is a finding about the measured code path, and it is meant to cover matching attempts
    /// appended after the finding as well as before it. What is refused is an invalidation that
    /// names evidence this campaign never wrote — a deletion with nothing to dispute.
    fn reaching_supersessions(
        lines: &[CampaignLedgerLine],
        identities: &[AttemptLines<'_>],
    ) -> Result<Vec<MethodSupersession>> {
        let mut supersessions = Vec::new();
        for line in lines {
            let CampaignRecord::Supersession { supersession } = line.body() else {
                continue;
            };
            ensure!(
                identities
                    .iter()
                    .any(|identity| supersession.covers(identity.key)),
                "the supersession at sequence {} is scoped to {:?}, which reaches no attempt this \
                 ledger records",
                line.seq().get(),
                supersession.scope(),
            );
            supersessions.push(supersession.clone());
        }
        Ok(supersessions)
    }

    /// Join every terminal record to its provisioning disposition, in the order the ledger wrote
    /// the terminal lines.
    ///
    /// Ledger order rather than inventory order, so `attempts()` displays the campaign as it
    /// actually unfolded — originals, failures and retries where they happened. Analysis
    /// reconstructs ladders by scale-point identity and never by this order.
    ///
    /// Infallible: every preceding pass has already established that each terminal line has a
    /// bucket, and that the bucket's provisioned line is present exactly when the outcome requires
    /// one.
    fn accounted_in_terminal_order(
        lines: &[CampaignLedgerLine],
        identities: &[AttemptLines<'_>],
    ) -> Vec<AccountedAttemptRecord> {
        let mut attempts = Vec::new();
        for line in lines {
            let CampaignRecord::Terminal { record } = line.body() else {
                continue;
            };
            let identity = identities
                .iter()
                .find(|identity| identity.key == record.key())
                .expect("every terminal line was bucketed under its own identity");
            attempts.push(AccountedAttemptRecord {
                terminal: record.clone(),
                provisioning: identity.provisioning(),
            });
        }
        attempts
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

// A *sibling* of `sealed`, never a child — which is the whole point of the topology this file's
// header describes. These tests build ledgers and reconcile them through the real constructor; they
// cannot write any of the three sealed struct literals, so an accounting rule they fail to exercise
// is a rule no test here can fake having passed.
#[cfg(test)]
mod tests;
