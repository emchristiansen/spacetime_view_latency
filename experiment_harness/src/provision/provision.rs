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

/// Provision one isolated server for `run`, run `body` against the *live* server and its
/// immutable manifest, and return `body`'s result.
///
/// Pipeline: resolve+verify the pinned distribution, stage the hash-verified WASM (fail fast
/// before starting anything if the built bytes don't match the committed provenance hash),
/// start the `/proc`-proven standalone, publish the staged bytes, snapshot the inert manifest,
/// and invoke `body(&server, &manifest)` while the server is alive — then, on **every** path,
/// attempt all outstanding teardowns (server shutdown and staged-file cleanup), aggregating the
/// primary error (whether from provisioning or from `body`) with every cleanup error. `body`'s
/// value is returned only if `body` succeeded *and* all cleanup succeeded. The live server and
/// manifest are borrowed for the duration of `body`, so no live capability can escape it.
pub(crate) fn provision_and_run<T>(
    listen: ListenAddress,
    module_wasm: &Path,
    run: RunCoordinate,
    seed: ScheduleSeed,
    body: impl FnOnce(&RunningPinnedServer, &ValidatedRunManifest) -> Result<T>,
) -> Result<T> {
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

    // Both `server` and `staged` are loud. Publish, assemble the manifest while the server is
    // alive (the manifest value-copies, retaining no handle), and run `body` — then tear both
    // down unconditionally.
    let outcome = publish_assemble_and_run(&distribution, &server, &staged, run, seed, body);

    let mut errors = Vec::new();
    let result = match outcome {
        Ok(value) => Some(value),
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

    match result {
        // `body` succeeded and every teardown succeeded.
        Some(value) if errors.is_empty() => Ok(value),
        // `body` succeeded but a teardown failed — surface the cleanup failure rather than a
        // value obtained from a run whose server/tempfile may have leaked.
        Some(_) | None => Err(into_error(errors)),
    }
}

/// Provision one isolated server for `run` and return its immutable [`ValidatedRunManifest`]
/// after teardown. The effectful provisioning smoke path: it provisions and snapshots but does
/// not measure. Implemented as the trivial [`provision_and_run`] body that clones the manifest.
pub(crate) fn provision_run(
    listen: ListenAddress,
    module_wasm: &Path,
    run: RunCoordinate,
    seed: ScheduleSeed,
) -> Result<ValidatedRunManifest> {
    provision_and_run(listen, module_wasm, run, seed, |_server, manifest| {
        Ok(manifest.clone())
    })
}

/// Publish the staged bytes, snapshot the inert manifest, and run `body`. Kept separate so any
/// failure flows through the driver's unconditional teardown rather than a bare `?` that would
/// strand the live server and staged tempfile.
fn publish_assemble_and_run<T>(
    distribution: &VerifiedDistribution,
    server: &RunningPinnedServer,
    staged: &StagedModuleWasm,
    run: RunCoordinate,
    seed: ScheduleSeed,
    body: impl FnOnce(&RunningPinnedServer, &ValidatedRunManifest) -> Result<T>,
) -> Result<T> {
    let artifact = server.publish(distribution, staged)?;
    let manifest = ValidatedRunManifest::assemble(run, distribution, server, &artifact, seed);
    body(server, &manifest)
}
