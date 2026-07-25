//! The Pilot-stage driver for the `EntityOwnerSenderView` candidate.
//!
//! Phase 1: the inventory is sealed, the ledger created, and every predeclared attempt walked. The
//! per-attempt body is `todo!()`. Nothing here provisions a server or measures anything yet.

use std::path::Path;

use anyhow::{Context, Result};

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::pilot_record::PilotRecord;
use crate::entity_owner_pilot::pilot_sink::PilotSink;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::output_path::OutputPath;

/// Run the whole Pilot: freeze the inventory from `seed`, create the ledger at `output`, and execute
/// every predeclared attempt in the frozen order, each on its own fresh isolated server.
///
/// The inventory is recorded before the first attempt executes, so the order the evidence is
/// interpreted against is on disk before any evidence exists.
///
/// An attempt's failure is not propagated: a subscription, reducer, or sample error terminates only
/// that attempt while later inventory attempts continue. Only a ledger failure stops the Pilot,
/// because past that point no result could be recorded honestly.
pub(crate) fn entity_owner_sender_view_pilot(
    listen: ListenAddress,
    module_wasm: &Path,
    output: &OutputPath,
    seed: ScheduleSeed,
) -> Result<()> {
    let inventory =
        AttemptInventory::frozen(seed).context("freezing the Pilot attempt inventory")?;

    let mut sink = PilotSink::create(output).context("creating the Pilot ledger")?;

    sink.write(&PilotRecord::Inventory {
        seed,
        inventory: &inventory,
    })
    .context("recording the frozen attempt inventory and block order")?;

    for attempt in inventory.attempts() {
        run_attempt(&mut sink, listen, module_wasm, seed, *attempt)?;
    }

    sink.finalize().context("finalizing the Pilot ledger")
}

/// Execute one predeclared attempt on its own fresh isolated server, appending each rung's evidence
/// as it confirms and exactly one terminal record when the attempt settles.
fn run_attempt(
    _sink: &mut PilotSink,
    _listen: ListenAddress,
    _module_wasm: &Path,
    _seed: ScheduleSeed,
    _attempt: AttemptKey,
) -> Result<()> {
    todo!("Phase 2: provision, seed the ladder progressively, measure each rung, and settle")
}
