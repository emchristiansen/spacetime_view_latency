//! A disposition must agree with how deep acquisition actually got, in both directions.

use anyhow::anyhow;

use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::indexed_sender_view_calibration_pilot::provision_depth::ProvisionDepth;
use crate::indexed_sender_view_calibration_pilot::resource_disposition::ResourceDisposition;

/// Coverage: the full matrix of acquisition depth against the three dispositions.
///
/// Both directions are defects and both are rejected, for different reasons:
///
/// - claiming a **release** where nothing releasable was ever acquired invents a cleanup that never
///   ran, which reads in the ledger as evidence the harness tidied up after itself;
/// - claiming **`NotAcquired`** after a module was staged or a server published silently drops the
///   fact that a tempfile, a process, and a data directory were this driver's to release — so a leak
///   would be recorded as nothing having existed.
///
/// The depth boundary is asserted rather than assumed: `acquired_releasable` is defined as
/// `has_staged_module`, so the transition sits between `DistributionResolved` and `ModuleStaged`.
/// That is load-bearing — resolving the distribution only reads paths out of the Nix store, and a
/// server-start failure adds nothing further because `RunningPinnedServer::start` reaps its own child
/// and data directory before returning. Pinning it here means a change to that premise breaks a test
/// rather than quietly changing what a ledger line claims.
#[test]
fn disposition_must_match_acquisition_depth() {
    let failed = ResourceDisposition::ReleaseFailed {
        diagnostic: DiagnosticArtifact::of_error(&anyhow!("teardown failed")),
    };

    for depth in ProvisionDepth::ALL {
        let acquired = depth.acquired_releasable();
        assert_eq!(
            acquired,
            depth.has_staged_module(),
            "{depth:?}: owning something releasable is exactly having staged the module"
        );

        let not_acquired = ResourceDisposition::NotAcquired.ensure_matches_acquisition(acquired);
        let released = ResourceDisposition::Released.ensure_matches_acquisition(acquired);
        let release_failed = failed.ensure_matches_acquisition(acquired);

        if acquired {
            not_acquired.expect_err(
                "a depth owning a releasable resource cannot claim nothing was acquired",
            );
            released.expect("a released claim is admissible once something was acquired");
            release_failed.expect("a failed release is admissible once something was acquired");
        } else {
            not_acquired.expect("nothing acquired is the only truthful claim at this depth");
            released.expect_err("a release cannot be claimed where nothing was acquired");
            release_failed.expect_err("a failed release cannot be claimed where nothing was acquired");
        }
    }

    // The boundary itself, stated once rather than inferred from the loop.
    assert!(!ProvisionDepth::NothingResolved.acquired_releasable());
    assert!(!ProvisionDepth::DistributionResolved.acquired_releasable());
    assert!(ProvisionDepth::ModuleStaged.acquired_releasable());
    assert!(ProvisionDepth::ServerStarted.acquired_releasable());

    // Only a failed release carries a diagnostic; the other two have nothing to report.
    assert!(ResourceDisposition::NotAcquired.failure_diagnostic().is_none());
    assert!(ResourceDisposition::Released.failure_diagnostic().is_none());
    assert!(failed.failure_diagnostic().is_some());
}
