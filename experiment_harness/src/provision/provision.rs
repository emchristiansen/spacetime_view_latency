//! The provisioning driver: turn a run coordinate into a validated run manifest.

use std::path::Path;

use anyhow::Result;

use crate::manifest::listen_address::ListenAddress;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;

/// Provision one isolated server for `run` and return its immutable [`ValidatedRunManifest`].
///
/// Pipeline: resolve+verify the pinned distribution, stage the hash-verified WASM (fail fast
/// before starting anything if the built bytes don't match the committed provenance hash),
/// start the `/proc`-proven standalone, publish the staged bytes, and snapshot the inert
/// manifest — then, on **every** path, attempt all outstanding teardowns (server shutdown and
/// staged-file cleanup), aggregating the primary error with every cleanup error. The manifest
/// is returned only if it was assembled *and* all cleanup succeeded.
pub(crate) fn provision_run(
    listen: ListenAddress,
    module_wasm: &Path,
    run: RunCoordinate,
    seed: ScheduleSeed,
) -> Result<ValidatedRunManifest> {
    let distribution = VerifiedDistribution::resolve()?;

    // Stage first: a hash mismatch must abort before any server is provisioned. No loud-Drop
    // capability is live before this succeeds, so a plain `?` cannot strand one.
    let staged = StagedModuleWasm::load(module_wasm, WasmSha256::new(MODULE_WASM_SHA256))?;

    // `staged` is loud from here. If the server fails to start, it is the only outstanding
    // teardown.
    let server = match RunningPinnedServer::start(&distribution, listen) {
        Ok(server) => server,
        Err(primary) => {
            let mut errors = vec![primary];
            if let Err(e) = staged.cleanup() {
                errors.push(e);
            }
            return Err(into_error(errors));
        }
    };

    // Both `server` and `staged` are loud. Assemble the manifest while the server is alive
    // (the manifest value-copies, retaining no handle), then tear both down unconditionally.
    let assembled = publish_and_assemble(&distribution, &server, &staged, run, seed);

    let mut errors = Vec::new();
    let manifest = match assembled {
        Ok(manifest) => Some(manifest),
        Err(primary) => {
            errors.push(primary);
            None
        }
    };
    if let Err(e) = server.shutdown() {
        errors.push(e);
    }
    if let Err(e) = staged.cleanup() {
        errors.push(e);
    }

    match manifest {
        // Assembled and every teardown succeeded.
        Some(manifest) if errors.is_empty() => Ok(manifest),
        // Assembled but a teardown failed — surface the cleanup failure rather than a manifest
        // whose server/tempfile may have leaked.
        Some(_) | None => Err(into_error(errors)),
    }
}

/// Publish the staged bytes and snapshot the inert manifest. Kept separate so its failure
/// flows through the driver's unconditional teardown rather than a bare `?` that would strand
/// the live server and staged tempfile.
fn publish_and_assemble(
    distribution: &VerifiedDistribution,
    server: &RunningPinnedServer,
    staged: &StagedModuleWasm,
    run: RunCoordinate,
    seed: ScheduleSeed,
) -> Result<ValidatedRunManifest> {
    let artifact = server.publish(distribution, staged)?;
    Ok(ValidatedRunManifest::assemble(
        run,
        distribution,
        server,
        &artifact,
        seed,
    ))
}
