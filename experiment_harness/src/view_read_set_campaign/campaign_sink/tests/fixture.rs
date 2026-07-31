//! Shared fixture: one real record of every campaign line shape a unit test can construct.
//!
//! Each record here comes from its own real constructor over real frozen constants — the frozen
//! inventory, the campaign's resolved pins, gate readings that genuinely pass and genuinely fail.
//! Nothing is fabricated, because the sink serializes whatever it is handed and a hand-shaped body
//! would prove the line format of a record the campaign never writes.
//!
//! **Five shapes, not six.** [`CampaignRecord::Provisioned`] carries an
//! [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance),
//! whose sole constructor reads a verified distribution, a running server, and a published module
//! artifact, so no record of that shape exists in this process. Its serialization is compile-checked
//! only; see [`super`].
//!
//! This tree keeps its own fixture rather than reaching into the reconciliation tree's, whose
//! helpers are `pub(super)` to that tree — the same reason the driver's tests keep their own. It is
//! also the honest boundary: a ledger-accounting fixture exists to be reconciled, and changes made
//! for that purpose must not silently move what this tree feeds the writer.

use crate::view_read_set_campaign::attempt_inventory::AttemptInventory;
use crate::view_read_set_campaign::attempt_key::AttemptKey;
use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
use crate::view_read_set_campaign::campaign_params::{
    ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
};
use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
use crate::view_read_set_campaign::campaign_record::CampaignRecord;
use crate::view_read_set_campaign::diagnostic_artifact::DiagnosticArtifact;
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;
use crate::view_read_set_campaign::failed_environment_gate::FailedEnvironmentGate;
use crate::view_read_set_campaign::method_supersession::MethodSupersession;
use crate::view_read_set_campaign::passed_environment_gate::PassedEnvironmentGate;
use crate::view_read_set_campaign::superseded_scope::SupersededScope;
use crate::view_read_set_campaign::supersession_justification::SupersessionJustification;
use crate::view_read_set_campaign::terminal_attempt_record::TerminalAttemptRecord;

/// One identity the frozen inventory really predeclares. Read out of the inventory rather than built
/// from coordinates, so it cannot drift from the campaign it claims to belong to.
pub(super) fn attempt() -> AttemptKey {
    AttemptInventory::frozen()
        .expect("the frozen campaign inventory has sixty distinct slots")
        .attempts()
        .first()
        .copied()
        .expect("the frozen inventory predeclares at least one attempt")
}

/// The opening line of every campaign: the frozen order and the pins it will be interpreted under.
pub(super) fn inventory_record() -> CampaignRecord {
    CampaignRecord::Inventory {
        inventory: AttemptInventory::frozen()
            .expect("the frozen campaign inventory has sixty distinct slots"),
        provenance: CampaignProvenance::resolved()
            .expect("the frozen version and release-commit pins parse"),
    }
}

/// The prospective clearance that lets one attempt launch.
pub(super) fn preflight_cleared_record() -> CampaignRecord {
    CampaignRecord::PreflightCleared {
        attempt: attempt(),
        gate: PassedEnvironmentGate::cleared(gate_evidence(50))
            .expect("a load that falls well under four CPUs passes every clause of the gate"),
    }
}

/// One attempt's terminal disposition — a preflight rejection, the outcome class that needs no
/// provisioned instance at all.
pub(super) fn terminal_record() -> CampaignRecord {
    CampaignRecord::Terminal {
        record: TerminalAttemptRecord::sealed(attempt(), preflight_rejected())
            .expect("a refused gate is a terminal outcome carrying no scale-bound evidence"),
    }
}

/// The host reading taken immediately after a measured attempt.
pub(super) fn post_attempt_record() -> CampaignRecord {
    CampaignRecord::PostAttemptEnvironment {
        attempt: attempt(),
        sample: EnvironmentSample::observed(
            ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
            50,
            ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
            0,
            0,
        ),
    }
}

/// An appended finding that one attempt's evidence is no longer method-valid.
pub(super) fn supersession_record() -> CampaignRecord {
    CampaignRecord::Supersession {
        supersession: MethodSupersession::recorded(
            SupersededScope::Attempt(attempt()),
            SupersessionJustification::parsed("the fixture's recorded invalidation")
                .expect("a nonempty justification parses"),
        ),
    }
}

/// Every record shape a unit test can construct, in a fixed order, paired with the variant name
/// serde's external tagging must emit for it.
pub(super) fn purely_constructible_records() -> Vec<(CampaignRecord, &'static str)> {
    vec![
        (inventory_record(), "Inventory"),
        (preflight_cleared_record(), "PreflightCleared"),
        (terminal_record(), "Terminal"),
        (post_attempt_record(), "PostAttemptEnvironment"),
        (supersession_record(), "Supersession"),
    ]
}

/// Assert that one emitted ledger line is exactly the `{seq, body}` object the sink promises.
///
/// Shared by the two tests that read the sink's output — one from the writer seam, one back off
/// disk — so neither can drift in what "one line" is asserted to mean.
///
/// `expected_body` is the record's own `serde_json` value, computed by the caller *before* it handed
/// the record over. That is what makes this an equality rather than a spot check: the sink may pair a
/// body with a sequence, and may do nothing else to it. A line that carried the right variant tag but
/// a truncated, altered, or enriched body would pass every weaker assertion and be wrong.
///
/// The comparison is [`serde_json::Value`] equality — *structural*, not bytewise. Two renderings
/// that differ only in object member order or in which characters they escape are the same value,
/// and this asserts the value rather than the encoding. The top-level key set is the one thing
/// checked exactly, because "and no other field" is a claim about the object, not about its
/// rendering.
pub(super) fn assert_ledger_line(
    index: usize,
    line: &[u8],
    expected_seq: u64,
    expected_body: &serde_json::Value,
    expected_kind: &str,
) {
    let parsed: serde_json::Value =
        serde_json::from_slice(line).expect("each ledger line must be one JSON object");
    let object = parsed
        .as_object()
        .expect("each ledger line must be a JSON object");

    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["body", "seq"],
        "line {index} must carry exactly the sequence and the body, and no other field"
    );

    assert_eq!(
        object["seq"].as_u64(),
        Some(expected_seq),
        "line {index} must carry the sequence the sink assigned it"
    );
    assert_eq!(
        object["body"], *expected_body,
        "line {index} must carry a body structurally equal to the record's own JSON value"
    );

    let body = object["body"]
        .as_object()
        .expect("the body is serde-externally-tagged, so it is a one-key object");
    let tags: Vec<&str> = body.keys().map(String::as_str).collect();
    assert_eq!(
        tags,
        [expected_kind],
        "line {index} must be tagged with exactly its record kind"
    );
}

/// An attempt the prospective gate refused to launch, because the load did not fall.
fn preflight_rejected() -> AttemptOutcome {
    AttemptOutcome::PreflightRejected {
        gate: FailedEnvironmentGate::refused(gate_evidence(100))
            .expect("a load that does not fall between the two samples fails the gate"),
        diagnostic: DiagnosticArtifact::of_error(&anyhow::anyhow!(
            "the fixture's terminating condition"
        )),
    }
}

/// Two readings the frozen separation apart on a four-CPU host, differing only in the second's load
/// — the fixture's single lever over whether the pair passes.
fn gate_evidence(second_load_centi: u64) -> EnvironmentGateEvidence {
    let first = EnvironmentSample::observed(0, 100, ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES, 0, 0);
    let second = EnvironmentSample::observed(
        ENVIRONMENT_SAMPLE_SEPARATION_NANOS,
        second_load_centi,
        ENVIRONMENT_MIN_AVAILABLE_RAM_BYTES,
        0,
        0,
    );
    EnvironmentGateEvidence::paired(first, second, 4)
        .expect("the two readings are the frozen separation apart on a host reporting four CPUs")
}
