//! The provisioning driver: turn a run coordinate into a validated run manifest.

use std::path::Path;

use anyhow::Result;

use crate::manifest::listen_address::ListenAddress;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::provision::provisioned_run::ProvisionedRun;
use crate::provision::run_resources::RunResources;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;
use crate::provision::verified_distribution::VerifiedDistribution;

/// Provision one isolated server for `run`, run `body` against the *live* server and its immutable
/// manifest, and return `body`'s result — the borrowed-capability adapter over the
/// [`provision_run_resources`] ownership-transfer primitive.
///
/// Acquires the resources through [`provision_run_resources`] — the single acquisition path, with
/// all partial-failure teardown — then borrows the live server and immutable manifest to `body` and,
/// on **every** path, unconditionally tears the acquired [`RunResources`] down, aggregating `body`'s
/// error (if any) with any teardown error. `body`'s value is returned only if `body` succeeded *and*
/// the teardown succeeded, so the live server and manifest cannot escape `body` and no value from a
/// run whose server/tempfile may have leaked is surfaced as success.
///
/// Panic semantics are unchanged from the pre-adapter version and no `catch_unwind` intervenes: a
/// panic in `body` unwinds through the owned `resources`, and because [`RunResources`] has no `Drop`
/// its members' own loud `Drop` guards fire on that unwinding drop — asserting the un-torn-down
/// server and staged WASM exactly as two bare locals would have.
pub(crate) fn provision_and_run<T>(
    listen: ListenAddress,
    module_wasm: &Path,
    run: RunCoordinate,
    seed: ScheduleSeed,
    body: impl FnOnce(&RunningPinnedServer, &ValidatedRunManifest) -> Result<T>,
) -> Result<T> {
    let (resources, manifest) =
        provision_run_resources(listen, module_wasm, run, seed)?.into_parts();

    // Borrow the live server and immutable manifest to `body`, then tear the resources down
    // unconditionally and aggregate `body`'s error (if any) with any teardown error.
    let outcome = body(resources.server(), &manifest);

    let mut errors = Vec::new();
    let result = match outcome {
        Ok(value) => Some(value),
        Err(primary) => {
            errors.push(primary);
            None
        }
    };
    if let Err(e) = resources.teardown() {
        errors.push(e);
    }

    match result {
        // `body` succeeded and the teardown succeeded.
        Some(value) if errors.is_empty() => Ok(value),
        // `body` failed, or `body` succeeded but the teardown failed — surface the aggregated error
        // rather than a value from a run whose server/tempfile may have leaked.
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

/// Provision one isolated server for `run` and hand its live resources onward as an owned
/// [`ProvisionedRun`], instead of tearing them down at scope exit. The single resource-acquisition
/// primitive: [`provision_and_run`] is the borrowed-capability adapter layered over it.
///
/// The pipeline — resolve+verify the pinned distribution, stage the hash-verified WASM (fail fast
/// before starting anything), start the `/proc`-proven standalone, then publish and snapshot the
/// immutable manifest — is the sole place these resources are acquired, so its partial-failure
/// teardown ladder exists in exactly one implementation and cannot drift against a sibling. On the
/// **success** path the live [`RunningPinnedServer`] and [`StagedModuleWasm`] are bundled into
/// [`RunResources`] and returned inside a [`ProvisionedRun`] rather than shut down: ownership of
/// both loud capabilities *escapes* this scope by move, to be owned for the duration of measurement
/// by the run's linear cleanup owner ([`RunCleanup`](crate::campaign::run_cleanup::RunCleanup)),
/// which performs the obligatory teardown exactly once at settle. Every path that does **not** hand
/// ownership onward still tears down whatever loud capability is live, aggregating each error, so
/// nothing live is ever stranded.
pub(crate) fn provision_run_resources(
    listen: ListenAddress,
    module_wasm: &Path,
    run: RunCoordinate,
    seed: ScheduleSeed,
) -> Result<ProvisionedRun> {
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

    // Both `server` and `staged` are loud. Publish and assemble the manifest while the server is
    // alive (the manifest value-copies, retaining no handle). On success, ownership of both
    // resources escapes by move via `ProvisionedRun` — no scope-exit teardown. On failure, tear
    // both down unconditionally, mirroring `provision_and_run`'s discipline.
    match server.publish(&distribution, &staged) {
        Ok(artifact) => {
            let manifest =
                ValidatedRunManifest::assemble(run, &distribution, &server, &artifact, seed);
            Ok(ProvisionedRun::new(
                RunResources::new(server, staged),
                manifest,
            ))
        }
        Err(primary) => {
            let mut errors = vec![primary];
            if let Err(e) = server.shutdown() {
                errors.push(e);
            }
            if let Err(e) = staged.cleanup() {
                errors.push(e);
            }
            Err(into_error(errors))
        }
    }
}
