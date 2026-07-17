//! The block's paired run carrier: drive the run bound by a predecessor-minted [`BlockRunDraw`].

use std::path::Path;

use crate::campaign::campaign_aborted::CampaignAborted;
use crate::campaign::finalization_outcome::FinalizationOutcome;
use crate::campaign::run_acquisition_failure::RunAcquisitionFailure;
use crate::campaign::run_cleanup::ConnectFailure;
use crate::campaign::run_cleanup::ResolutionFailure;
use crate::campaign::run_cleanup::RunCleanup;
use crate::campaign::run_cursor::RunSettled;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::campaign::run_driver::DriveRun;
use crate::campaign::run_driver::RunDriver;
use crate::client::connected_client::ConnectedClient;
use crate::dataset::run_dataset::RunDataset;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::provision::provision::provision_run_resources;
use crate::roles::role_identities::RoleIdentities;

use super::block_ready::BlockRunDraw;
use super::BlockRunOutcome;

/// The paired carrier a block hands out for its next run. It owns exactly one opaque [`BlockRunDraw`] — the
/// predecessor-minted binding of the run's writing state, single-source coordinate, canonical seed, and the
/// block's private remainder. It accepts **no** identity-bearing loose arguments: [`Self::from_draw`] takes
/// only the whole draw, so no caller can pair this run with a foreign coordinate, seed, or remainder.
/// [`Self::drive`] decomposes the draw and runs the concrete provision → connect → resolve → arm →
/// [`RunDriver::drive`] effect path *itself* (see [`acquire_and_drive`]), then folds the resulting run
/// terminal back into the block through the draw's own continuation.
///
/// Because the draw is minted only inside [`BlockReady::next_run`](super::BlockReady) from that block's own
/// fields, the run terminal folded in is always this run's own — the run/continuation coordinate match is
/// structural, not a runtime `assert_eq!` that could unwind across the unfinalized sink. The carrier accepts
/// only non-identity server/Wasm configuration ([`ListenAddress`] and the module WASM path) at
/// [`Self::drive`]; it never accepts an effect closure, an acquired identity, resolved resources, or a run
/// terminal, so no foreign identity or terminal can be injected.
pub(crate) struct BlockRunPending {
    draw: BlockRunDraw,
}

impl BlockRunPending {
    /// Wrap the predecessor-minted draw. Accepts only the opaque [`BlockRunDraw`] and nothing else, so there
    /// is no decomposed-piece seam here. `pub(in crate::campaign::block_cursor)` so only the block cursor can
    /// build a carrier; the sole implemented caller is [`BlockReady::next_run`](super::BlockReady), which
    /// hands the draw it just minted.
    pub(in crate::campaign::block_cursor) fn from_draw(draw: BlockRunDraw) -> Self {
        Self { draw }
    }

    /// Decompose the draw and acquire, drive, and settle this exact run, then fold its terminal back into the
    /// block. Runs the concrete effect path from the draw's own coordinate and seed (see
    /// [`acquire_and_drive`]); on a completed run resumes the block on its own iterator, on a stopped run
    /// aborts the block at its frontier. A pre-cleanup acquisition failure short-circuits on the Err channel
    /// as a [`CampaignAborted`] whose sink is already finalized — no block terminal is minted in that case.
    /// `pub(in crate::campaign)` confines this to the campaign module subtree — not one caller, which the
    /// visibility does not single out; the sole implemented caller is the campaign-block carrier
    /// ([`CampaignBlockPending::drive`](crate::campaign::campaign_cursor::CampaignBlockPending)).
    pub(in crate::campaign) fn drive(
        self,
        listen: ListenAddress,
        module_wasm: &Path,
    ) -> Result<BlockRunOutcome, CampaignAborted> {
        let (writing, coordinate, continuation) = self.draw.into_parts();
        let settled = acquire_and_drive(
            listen,
            module_wasm,
            writing,
            coordinate,
            continuation.seed(),
        )?;
        Ok(match settled {
            RunSettled::Done(done) => BlockRunOutcome::Resumed(continuation.finish(done)),
            RunSettled::Incomplete(incomplete) => {
                BlockRunOutcome::Aborted(continuation.abort(incomplete))
            }
        })
    }
}

/// Run the concrete provision → connect → resolve → arm → drive acquisition path for one run and settle it,
/// or abort the campaign if acquisition fails before the run's cleanup owner is armed.
///
/// Provision the run's isolated resources for the single-source `coordinate` — which provisioning moves in
/// and assembles the manifest from, so the manifest's coordinate *is* that value by construction — then
/// connect the measured client, resolve its authenticated role identities and bind the [`RunDataset`], and
/// only then arm the infallible [`RunCleanup`] from the already-connected client and resources and drive an
/// infallible [`RunDriver`] to a settled terminal in one expression. Provisioning, connect, and identity
/// resolution are the fallible steps, and all three occur *before* a `RunCleanup` (the cleanup "bomb")
/// exists; on any of them recover the entry sink from `writing` via
/// [`RunWritingManifest::abandon`](crate::campaign::run_cursor::RunWritingManifest::abandon), finalize it,
/// and return a typed [`CampaignAborted`] retaining the acquisition failure and the finalization outcome.
/// Once the `RunCleanup` is armed, no fallible step runs while it is owned unsettled — the driver settles it
/// on every branch — so the returned [`RunSettled`] is always the product of real cleanup.
///
/// No post-provision coordinate assertion guards this: an `assert_eq!` here would run while the loud
/// [`RunResources`](crate::provision::run_resources::RunResources) guards and the unfinalized entry sink
/// are both live, so a fired assert would unwind across those guards and drop the sink unfinalized — the
/// exact ownership hole this design forbids. The single-source coordinate already makes a mismatch
/// unrepresentable.
fn acquire_and_drive(
    listen: ListenAddress,
    module_wasm: &Path,
    writing: RunWritingManifest,
    coordinate: RunCoordinate,
    seed: ScheduleSeed,
) -> Result<RunSettled, CampaignAborted> {
    // The run's cell is needed to derive the growth role identity after connection; take it before
    // provisioning moves the coordinate in.
    let cell = coordinate.cell();

    // Provision the run's isolated server + staged WASM. A provisioning failure happens before any cleanup
    // owner exists and before any client connected, so recover and finalize the sink and abort.
    let provisioned = match provision_run_resources(listen, module_wasm, coordinate, seed) {
        Ok(provisioned) => provisioned,
        Err(failure) => {
            return Err(abort_acquisition(
                writing,
                RunAcquisitionFailure::Provision(failure),
            ));
        }
    };
    let (resources, manifest) = provisioned.into_parts();

    // Connect the measured client — the first step that could leave a live client, but still before any
    // cleanup owner. A connect failure tears the resources down here and retains the connect error paired
    // with that teardown result; no client survives, so recover and finalize the sink and abort.
    let server_url = resources.server().listen().client_url();
    let database_identity = manifest.database_identity().identity().to_hex().to_string();
    let client = match ConnectedClient::connect(&server_url, &database_identity) {
        Ok(client) => client,
        Err(connect_error) => {
            let teardown = resources.teardown();
            return Err(abort_acquisition(
                writing,
                RunAcquisitionFailure::Connect(ConnectFailure::new(
                    connect_error.context("connecting the measured subscriber"),
                    teardown,
                )),
            ));
        }
    };

    // Resolve the authenticated measured identity and bind the run's dataset while the cleanup owner is
    // still un-armed. A resolution failure owns both a disconnect and a teardown obligation — attempt both,
    // retaining all three results — then recover and finalize the sink and abort. Doing this before arming
    // is what keeps the cleanup "bomb" from being stranded across this fallible step.
    let identities = match RoleIdentities::resolve(client.measured_identity(), seed, cell) {
        Ok(identities) => identities,
        Err(resolve_error) => {
            let disconnect = client.disconnect();
            let teardown = resources.teardown();
            return Err(abort_acquisition(
                writing,
                RunAcquisitionFailure::Resolve(ResolutionFailure::new(
                    resolve_error,
                    disconnect,
                    teardown,
                )),
            ));
        }
    };
    let context = RunDataset::resolve(manifest, &identities);

    // Every fallible acquisition step has succeeded. Arm the cleanup "bomb" infallibly from the
    // already-connected client and resources, then build the infallible driver and drive it in one
    // expression: no `?` or other early return can strand the owned `RunCleanup`, and `drive` settles it on
    // every branch.
    let cleanup = RunCleanup::arm(client, resources);
    Ok(RunDriver::new(cleanup, writing, context).drive())
}

/// Recover the campaign's owned sink from the run's manifest-writing entry state, finalize it, and bundle
/// the typed acquisition `failure` with that finalization's outcome into a [`CampaignAborted`]. Called only
/// on a pre-cleanup acquisition failure, where no cleanup owner ever existed — so there is no honest run
/// terminal to mint — but the sink carrying prior runs' durable progress must still be sealed rather than
/// dropped. Mirrors the always-attempted finalize on the block-abort path.
fn abort_acquisition(
    writing: RunWritingManifest,
    failure: RunAcquisitionFailure,
) -> CampaignAborted {
    let sink = writing.abandon();
    let finalization = match sink.finalize() {
        Ok(()) => FinalizationOutcome::Sealed,
        Err(error) => FinalizationOutcome::Failed(error),
    };
    CampaignAborted::new(failure, finalization)
}
