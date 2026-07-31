//! Shared fixture: the real campaign pins, an agreeing set of observations, and values that differ.
//!
//! The agreeing observations are built from the same frozen constants
//! [`CampaignProvenance::resolved`] reads. **That is what these tests can prove and all they can
//! prove**: that each field is compared against the pin it names, and that agreement is reported as
//! agreement. Whether `2.7.0`, that release commit, and that module digest are the *externally
//! correct* things to have pinned is not a question any in-process test can answer — it is settled
//! by the preregistration and by the verification the harness performs against a real distribution.
//!
//! Nothing here provisions anything, and nothing constructs an
//! [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance):
//! these are comparison inputs, not evidence.

use semver::Version;

use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::params::{EXPECTED_RELEASE_COMMIT, EXPECTED_VERSION};
use crate::view_read_set_campaign::campaign_params::CONFIRMED_READS;
use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;
use crate::view_read_set_campaign::observed_runtime_pins::ObservedRuntimePins;

/// The campaign's real pins, resolved exactly as a running campaign resolves them.
///
/// Reachable in a test because resolution is pure: it parses frozen constants and reads
/// compile-time build environment, and touches no server, socket, or file.
pub(super) fn campaign() -> CampaignProvenance {
    CampaignProvenance::resolved().expect("the frozen version and release-commit pins parse")
}

/// The pinned runtime version, parsed through the same parser the pins go through.
pub(super) fn pinned_version() -> Version {
    Version::parse(EXPECTED_VERSION).expect("EXPECTED_VERSION is the frozen semver pin")
}

/// A well-formed version that differs from the pin, derived by perturbing the pin itself.
///
/// Every "other" value in this fixture is a perturbation of the current pin rather than a literal,
/// so it stays a mismatch when a pin is later changed. A hardcoded literal could one day *become*
/// the pin, and the isolated-mismatch tests would then quietly assert nothing.
pub(super) fn other_version() -> Version {
    let mut other = pinned_version();
    other.major = other
        .major
        .checked_add(1)
        .expect("the pinned major version is far below the top of its range");
    other
}

/// A well-formed release commit that differs from the pin, derived by flipping one bit of its first
/// byte.
///
/// Round-trips through the real canonical-hex parser, so the value is one the production path would
/// accept — a mismatch, not a malformed input.
pub(super) fn other_release_commit() -> ReleaseCommit {
    let pinned = ReleaseCommit::parse(EXPECTED_RELEASE_COMMIT)
        .expect("EXPECTED_RELEASE_COMMIT is the frozen canonical hex pin");
    let mut bytes = *pinned.bytes();
    bytes[0] ^= 0x01;
    ReleaseCommit::parse(&hex::encode(bytes))
        .expect("a one-bit perturbation of canonical hex is still canonical hex")
}

/// A well-formed module digest that differs from the pin, derived by flipping one bit of its first
/// byte.
pub(super) fn other_module_digest() -> WasmSha256 {
    let mut bytes = *WasmSha256::new(MODULE_WASM_SHA256).bytes();
    bytes[0] ^= 0x01;
    WasmSha256::new(bytes)
}

/// Observations that agree with the campaign on every one of the five compared facts.
///
/// Each test starts here and changes exactly one field, so what a failing test reports is always the
/// consequence of that one difference. Both version fields borrow the same value, which is correct:
/// the two binaries of one pinned distribution report the same version, and both are compared
/// against the single `expected_version` pin.
pub(super) fn agreeing(pinned_version: &Version) -> ObservedRuntimePins<'_> {
    ObservedRuntimePins {
        cli_version: pinned_version,
        standalone_version: pinned_version,
        cli_release_commit: ReleaseCommit::parse(EXPECTED_RELEASE_COMMIT)
            .expect("EXPECTED_RELEASE_COMMIT is the frozen canonical hex pin"),
        module_wasm_sha256: WasmSha256::new(MODULE_WASM_SHA256),
        confirmed_reads: CONFIRMED_READS,
    }
}
