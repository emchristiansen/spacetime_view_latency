//! A release failure stays on the attempt that produced it, and every remaining slot settles.

use anyhow::anyhow;

use crate::control_registry_discovery_screen::attempt_failure::AttemptFailure;
use crate::control_registry_discovery_screen::attempt_inventory::{
    AttemptInventory, SCREEN_ATTEMPT_COUNT,
};
use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::partial_evidence::PartialEvidence;
use crate::control_registry_discovery_screen::partial_provision::PartialProvision;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::control_registry_discovery_screen::supersession::Supersession;
use crate::entity_owner_pilot::attempt_provenance::{DistributionFacts, ServerFacts};
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;

use super::super::settle_remaining;

/// A diagnostic distinctive enough to find in serialized output.
const RELEASE_DIAGNOSTIC: &str = "the server child process may still be running";

/// A distinct diagnostic for the gate-inoperable path, so the two stops cannot be confused.
const GATE_DIAGNOSTIC: &str = "the host waiter could not be spawned";

/// Coverage: a terminal stop keeps the sixteen-original invariant, and the stopping diagnostic is
/// durable on both the producing record and every skipped slot.
///
/// The failing release is recorded in **two** places on purpose, and neither is redundant. It is on
/// the producing attempt's own record, which is the only place it can live when the failure happens
/// on the final slot and no successor remains to explain it; and on each remaining slot, so a reader
/// learns why those were skipped without locating and interpreting another record. The driver reads
/// its stop decision off the record it just appended, so the decision and its durable evidence are
/// the same value.
///
/// The producing record is built at `ServerStarted`, not `NothingResolved`, because a failed release
/// is only *representable* where something was acquired — publication is the first boundary that
/// leaves the driver owning both a started server and a staged module. Pairing a failed release with
/// an empty provisioning prefix is refused by the acquisition cross-check, which is the correct
/// behaviour and is proven separately.
///
/// Both stop reasons run through the same table, differing only in where their suffix begins: a
/// failed release skips the slots *after* the producing one, while an inoperable gate never gated the
/// current slot either, so its suffix is the whole remaining inventory including that slot.
///
/// The suffix is checked by **serialized identity**, not merely by count: a settlement producing the
/// right number of records for the wrong slots would leave predeclared identities missing while
/// looking complete, which is precisely what "report generation fails on missing or duplicate planned
/// identities" must be able to catch.
#[test]
fn release_failure_settles_every_remaining_slot() {
    let seed = ScheduleSeed::new(7);
    let inventory = AttemptInventory::frozen(seed).expect("the seed freezes");
    let pinned = PinnedArtifactIdentity::frozen().expect("the pinned constants are well formed");
    let attempts = inventory.attempts();
    assert_eq!(attempts.len(), SCREEN_ATTEMPT_COUNT);

    let diagnostic = DiagnosticArtifact::of_error(&anyhow!(RELEASE_DIAGNOSTIC));

    // The producing attempt: its own record carries the release failure.
    let produced = ScreenRecord::not_provisioned(
        attempts[0],
        &pinned,
        seed,
        None,
        server_started(),
        provisioning_failure(),
        ResourceDisposition::ReleaseFailed {
            diagnostic: diagnostic.clone(),
        },
    )
    .expect("a server-started provisioning failure with a failed release is recordable");

    assert!(
        produced.release_failure_diagnostic().is_some(),
        "the driver must be able to read the release failure off the record it just appended"
    );
    let rendered = serde_json::to_string(&produced).expect("a record serializes");
    assert!(
        rendered.contains(RELEASE_DIAGNOSTIC),
        "the producing record must retain the failing release verbatim, got {rendered}"
    );

    // A release failure on the *final* slot has no successor to carry it, which is exactly why the
    // producing record does.
    let last = ScreenRecord::not_provisioned(
        attempts[SCREEN_ATTEMPT_COUNT - 1],
        &pinned,
        seed,
        None,
        server_started(),
        provisioning_failure(),
        ResourceDisposition::ReleaseFailed {
            diagnostic: diagnostic.clone(),
        },
    )
    .expect("the final slot records its own failed release");
    let empty_suffix = settle_remaining(
        &attempts[SCREEN_ATTEMPT_COUNT..],
        &pinned,
        seed,
        &NotRunReason::PriorAttemptReleaseFailed {
            diagnostic: diagnostic.clone(),
        },
    )
    .expect("settling an empty remainder succeeds");
    assert!(
        empty_suffix.is_empty(),
        "there is no slot after the last one to settle"
    );
    let rendered = serde_json::to_string(&last).expect("a record serializes");
    assert!(
        rendered.contains(RELEASE_DIAGNOSTIC),
        "the final slot's own record is the only place its failed release can be retained, got \
         {rendered}"
    );

    // Both stop reasons, through the one settlement helper.
    let stops: [(NotRunReason, &str, &[AttemptKey]); 2] = [
        (
            NotRunReason::PriorAttemptReleaseFailed { diagnostic },
            RELEASE_DIAGNOSTIC,
            &attempts[1..],
        ),
        (
            NotRunReason::GateInoperable {
                diagnostic: DiagnosticArtifact::of_error(&anyhow!(GATE_DIAGNOSTIC)),
            },
            GATE_DIAGNOSTIC,
            attempts,
        ),
    ];

    for (reason, expected_diagnostic, suffix) in stops {
        let settled =
            settle_remaining(suffix, &pinned, seed, &reason).expect("every suffix slot settles");
        assert_suffix_settled(&settled, suffix, expected_diagnostic);
    }

    // One producing record plus the release suffix is the full frozen inventory: stopping early
    // never leaves a predeclared slot without a terminal record.
    assert_eq!(attempts[1..].len() + 1, SCREEN_ATTEMPT_COUNT);
}

/// The deepest pre-publication prefix, where a failed release is representable: the server started
/// and the staged module is still the driver's to clean up.
fn server_started() -> PartialProvision {
    PartialProvision::ServerStarted {
        distribution: DistributionFacts::fixture(),
        staged_wasm_sha256: WasmSha256::new(MODULE_WASM_SHA256),
        server: ServerFacts::fixture(),
    }
}

/// A publication failure, rebuilt per use because `AttemptFailure` is consumed by its record.
fn provisioning_failure() -> AttemptFailure {
    AttemptFailure::observed(
        FailureKind::Provision,
        PartialEvidence::NothingObserved,
        DiagnosticArtifact::of_error(&anyhow!("publication was rejected")),
    )
    .expect("a provisioning failure is recordable")
}

/// Assert a settled run matches its expected suffix by serialized identity, shape, supersession, and
/// retained diagnostic.
fn assert_suffix_settled(
    settled: &[ScreenRecord],
    expected_keys: &[AttemptKey],
    expected_diagnostic: &str,
) {
    assert_eq!(
        settled.len(),
        expected_keys.len(),
        "every slot in the suffix must receive exactly one terminal record"
    );

    for (record, expected_key) in settled.iter().zip(expected_keys) {
        let ScreenRecord::NotRun {
            key, supersession, ..
        } = record
        else {
            panic!("a settled slot is NotRun: it acquired nothing and measured nothing");
        };
        assert_eq!(
            *key, *expected_key,
            "settlement must record the frozen identity of the slot it stands for, in order"
        );
        // This screen mints no retry, so every record it writes — settled or not — is an original.
        assert_eq!(
            *supersession,
            Supersession::Original,
            "a settled original slot supersedes nothing"
        );
        assert!(
            record.release_failure_diagnostic().is_none(),
            "a slot that never ran acquired nothing, so it reports no release of its own"
        );

        // The durable form is what a reader actually gets, so the identity, the absent supersession
        // link, and the stopping diagnostic are all confirmed in the serialized record.
        let rendered = serde_json::to_string(record).expect("a record serializes");
        let serialized_key = serde_json::to_string(expected_key).expect("a key serializes");
        assert!(
            rendered.contains(&serialized_key),
            "the settled record must serialize the frozen identity it stands for; expected \
             {serialized_key} in {rendered}"
        );
        assert!(
            rendered.contains("\"supersession\":\"Original\""),
            "a settled original must serialize with no supersession link, got {rendered}"
        );
        assert!(
            rendered.contains(expected_diagnostic),
            "a settled slot must name the diagnostic that stopped the run, got {rendered}"
        );
    }
}
