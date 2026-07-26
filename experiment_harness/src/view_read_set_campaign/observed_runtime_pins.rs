//! The five campaign-wide facts one attempt is admitted against.
//!
//! **Deliberately not sealed, because this is not evidence.** Every other invariant-bearing type in
//! this campaign hides its fields in a private childless `sealed` module so that a sole constructor
//! is the only door. This one does the opposite on purpose: its fields are named and crate-visible,
//! it holds no invariant, and constructing one asserts nothing about any server that ever ran. It is
//! an argument bundle — the inputs to a comparison, assembled at the call site and consumed
//! immediately.
//!
//! That is not a weakening, because minting one of these is not a capability. The admission gate's
//! authority rests on
//! [`ReconciledCampaign`](crate::view_read_set_campaign::reconciled_campaign::ReconciledCampaign)
//! accepting only an
//! [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance),
//! whose fields stay private to its own childless module and whose sole constructor still copies
//! them out of live capabilities. No value of this type can become one, be deserialized into one, or
//! stand in for one. What it makes possible is testing the comparison without provisioning a server
//! — which is why the comparison lives here rather than inside that sealed module, where no sibling
//! test could reach it and no test could reach a constructor at all.

use anyhow::{ensure, Result};
use semver::Version;

use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;

/// The five recorded facts of one provisioned attempt that have a campaign-wide pin to be checked
/// against.
///
/// **Why exactly five.** These are the facts every attempt of a campaign must share, so a
/// disagreement means the ledger mixes runtimes or builds. The rest of what
/// [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance)
/// records — PID, resolved executable, listen address and client URL, data and store directories,
/// raw version lines, and the published database identity — is retained diagnostics with no
/// campaign-wide equality pin. The database identity in particular is *expected* to differ per
/// attempt, since every scale point publishes a fresh instance, so comparing it would reject the
/// protocol the campaign runs.
///
/// A borrowing view rather than an owning record, because it exists for the duration of one
/// comparison. Nothing stores it, and nothing should: a retained copy would be a second place the
/// same facts live, free to drift from the record they were projected out of.
pub(crate) struct ObservedRuntimePins<'a> {
    /// The version the attempt's CLI binary reported.
    pub(crate) cli_version: &'a Version,
    /// The version the attempt's standalone server binary reported.
    pub(crate) standalone_version: &'a Version,
    /// The upstream release commit the attempt's CLI binary reported.
    pub(crate) cli_release_commit: ReleaseCommit,
    /// The digest of the module artifact the attempt actually published.
    pub(crate) module_wasm_sha256: WasmSha256,
    /// The confirmed-read setting the attempt's measurement ran under.
    pub(crate) confirmed_reads: bool,
}

impl ObservedRuntimePins<'_> {
    /// Check these observed facts against the campaign's pins, failing loud and naming both sides on
    /// any disagreement.
    ///
    /// Takes one whole [`CampaignProvenance`] rather than five loose expected values, so an attempt
    /// cannot be checked against one campaign's version and another's module digest.
    ///
    /// **Which clauses are real observations, and which is not.** The two versions, the release
    /// commit, and the module digest were read off a live distribution and a published artifact, so
    /// those four can genuinely disagree and are the substance of the gate. `confirmed_reads` cannot
    /// disagree *within one process*, since the attempt record and the campaign parameters both read
    /// [`CONFIRMED_READS`](crate::view_read_set_campaign::campaign_params::CONFIRMED_READS); what it
    /// catches is a ledger assembled from lines written by two different builds, which a
    /// ledger-only reader has no other way to detect.
    ///
    /// Every clause is a separate `ensure!` rather than one conjunction, so a failure says which
    /// fact disagreed and with what — the whole reconciliation fails on any disagreement, and a
    /// reader of that failure needs to know which pin moved.
    pub(crate) fn agrees_with(&self, campaign: &CampaignProvenance) -> Result<()> {
        ensure!(
            self.cli_version == campaign.expected_version(),
            "the attempt's cli reported version {} != the campaign's pinned {}",
            self.cli_version,
            campaign.expected_version(),
        );
        ensure!(
            self.standalone_version == campaign.expected_version(),
            "the attempt's standalone reported version {} != the campaign's pinned {}",
            self.standalone_version,
            campaign.expected_version(),
        );
        ensure!(
            self.cli_release_commit == campaign.expected_release_commit(),
            "the attempt's cli release commit {} != the campaign's pinned {}",
            self.cli_release_commit,
            campaign.expected_release_commit(),
        );
        ensure!(
            self.module_wasm_sha256 == campaign.expected_module_wasm_sha256(),
            "the attempt published module digest {} != the campaign's pinned {}",
            self.module_wasm_sha256.canonical_hex(),
            campaign.expected_module_wasm_sha256().canonical_hex(),
        );
        ensure!(
            self.confirmed_reads == campaign.parameters().confirmed_reads(),
            "the attempt measured with confirmed_reads={} but the campaign's parameters record {}; \
             this ledger mixes lines from two builds",
            self.confirmed_reads,
            campaign.parameters().confirmed_reads(),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests;
