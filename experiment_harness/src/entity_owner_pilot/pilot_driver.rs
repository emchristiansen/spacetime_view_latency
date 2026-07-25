//! The Pilot-stage driver for the `EntityOwnerSenderView` candidate.
//!
//! Phase 1 (this commit) establishes the type skeleton and the command wiring: the frozen inventory
//! is sealed, the durable ledger is created, and every predeclared attempt is walked. The
//! per-attempt provisioning/measurement body is `todo!()` and is filled in Phase 2 per the spec's
//! Parked Frontier. Nothing here provisions a server or measures anything yet.

use std::path::Path;

use anyhow::{Context, Result};

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::entity_owner_pilot::attempt_key::AttemptKey;
use crate::entity_owner_pilot::attempt_outcome::AttemptOutcome;
use crate::entity_owner_pilot::pilot_record::PilotRecord;
use crate::entity_owner_pilot::pilot_sink::PilotSink;
use crate::manifest::listen_address::ListenAddress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::output_path::OutputPath;

/// Run the whole `EntityOwnerSenderView` Pilot: freeze the attempt inventory from `seed`, create the
/// durable ledger at `output`, and execute every predeclared attempt in the frozen randomized order,
/// each on its own fresh isolated server bound to `listen`.
///
/// The inventory is frozen and durably recorded *before* the first attempt executes, so the block
/// order the evidence is interpreted against is on disk before any evidence exists. Every attempt
/// then produces exactly one terminal record — complete, failed with partial evidence, or not run
/// with a reason — which is what makes the ledger's per-attempt accounting checkable rather than
/// inferred from which lines happen to be present.
///
/// An attempt's failure is *not* propagated: the Parked Frontier requires that a subscription,
/// reducer, or sample error terminate only that attempt while "later inventory attempts continue".
/// Only a ledger failure stops the Pilot, because past that point no result could be recorded
/// honestly.
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

/// Execute one predeclared attempt end to end on its own fresh isolated server, appending each
/// rung's evidence as it confirms and exactly one terminal record when the attempt settles.
///
/// Phase 2 fills this in by composing the primitives the Smoke milestone already proved: the
/// [`crate::provision`] acquisition/teardown pipeline, [`crate::client::connected_client`]'s
/// candidate-specific seeding and subscription primitives, and a candidate-specific measured batch
/// sealing a [`RawLatencies`](crate::observation::raw_latencies::RawLatencies) per rung.
fn run_attempt(
    _sink: &mut PilotSink,
    _listen: ListenAddress,
    _module_wasm: &Path,
    _seed: ScheduleSeed,
    _attempt: AttemptKey,
) -> Result<()> {
    todo!("Phase 2: provision, seed the ladder progressively, measure each rung, and settle")
}

/// Append one attempt's single terminal disposition, binding it to the attempt identity so the
/// ledger's per-attempt accounting is complete even for attempts that produced no evidence.
fn record_terminal(
    sink: &mut PilotSink,
    attempt: AttemptKey,
    outcome: &AttemptOutcome,
) -> Result<()> {
    sink.write(&PilotRecord::Terminal { attempt, outcome })
        .map(|_id| ())
        .context("recording an attempt's terminal disposition")
}
