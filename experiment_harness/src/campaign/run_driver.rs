//! The real effectful run driver's ownership boundary: own the connected run, settle through cleanup.

use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::campaign::run_cleanup::RunCleanup;
use crate::campaign::run_cursor::RunAwaitingDose;
use crate::campaign::run_cursor::RunDoseStep;
use crate::campaign::run_cursor::RunManifestStep;
use crate::campaign::run_cursor::RunObservationStep;
use crate::campaign::run_cursor::RunSettled;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::client::connected_client::ConnectedClient;
use crate::dataset::dose_batch::DoseBatch;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::run_dataset::RunDataset;
use crate::observation::dose_event_counter::DoseEventCounter;
use crate::observation::dose_evidence::DoseEvidence;
use crate::observation::event_evidence::EventEvidence;
use crate::params::BATCH_DELAY_MS;

/// Linear owner of everything one provisioned, connected run needs to be driven to a cleanup-settled
/// terminal: the run's linear cleanup owner [`RunCleanup`] (the connected measured client plus the
/// provisioned resources), the run-cursor entry [`RunWritingManifest`] (which owns the observation sink),
/// and the bound [`RunDataset`] `context` — the run's immutable manifest paired with the dataset resolved
/// from that manifest's own coordinate. The role identities and dataset are resolved *before* this owner is
/// built (the connected client already exists), so no separate manifest field is held here: the manifest,
/// its coordinate, and the dataset are one owned value threaded through the cursor.
///
/// **Enforced boundary.** [`Self::new`] is the sole (infallible) constructor and [`DriveRun::drive`]
/// consumes this owner into `settle_executed`/`settle_stopped` on **every** branch, so the ownership below
/// is a compile-time property, not documentation: the campaign orchestrator connects the client (the sole
/// fallible step, failing into an acquisition-abort *before* a `RunCleanup` exists), then builds this owner
/// and drives it in one expression, never holding the cleanup "bomb" across a fallible step.
///
/// **Why `new` is infallible and `drive` never returns `Result`.** A `RunCleanup` holds a `MustDisconnect`
/// client guard whose `Drop` panics if the client was never taken; a panic while it is owned unwinds
/// through that guard, and a panic during unwinding aborts the process. So the moment this owner exists no
/// path may `?`-early-return or otherwise drop it un-settled. [`Self::new`] cannot fail, and
/// [`DriveRun::drive`] returns [`RunSettled`] (never a `Result` whose `?` would skip cleanup): every effect
/// failure is routed through a consuming run-cursor stop transition into
/// [`RunExecutionStopped`](crate::campaign::run_cursor::RunExecutionStopped) and then
/// [`RunCleanup::settle_stopped`], and dose exhaustion through
/// [`RunExecuted`](crate::campaign::run_cursor::RunExecuted) and [`RunCleanup::settle_executed`]. Both
/// consume the owned [`RunCleanup`], performing the ordered disconnect-then-teardown, so no branch can skip
/// settlement and none can double-consume (settle takes `self` by value).
///
/// **The `RunCleanup` terminal-minter property is enforced independently.** [`RunCleanup`] is the sole
/// minter of a run terminal, and neither it nor its `resources: RunResources` has a `Drop` (that is what
/// lets a consuming settle move their guards out without `unsafe`); the loud guards live on its
/// `MustDisconnect` client and on
/// [`RunResources`](crate::provision::run_resources::RunResources)'s members, so abandoning a `RunCleanup`
/// without settling still fires those guards rather than silently leaking.
pub(in crate::campaign) struct RunDriver {
    cleanup: RunCleanup,
    writing: RunWritingManifest,
    context: RunDataset,
}

impl RunDriver {
    /// Assemble the owner from the connected cleanup capability, the run-cursor entry state, and the bound
    /// run `context`. The run itself is not passed separately — it is read from the context manifest's
    /// coordinate ([`RunCoordinate::run`](crate::manifest::run_coordinate::RunCoordinate::run)) for the
    /// subscribed target. Infallible by construction: the fallible acquisition and identity-resolution steps
    /// happen in the campaign orchestrator *before* a [`RunCleanup`] exists, so nothing here can fail while
    /// the cleanup "bomb" is owned. `pub(in crate::campaign)` so only the orchestrator builds one.
    pub(in crate::campaign) fn new(
        cleanup: RunCleanup,
        writing: RunWritingManifest,
        context: RunDataset,
    ) -> Self {
        Self {
            cleanup,
            writing,
            context,
        }
    }
}

/// The run-driving contract: consume the [`RunDriver`] and drive its run to a settled terminal.
///
/// Returns [`RunSettled`] and never a `Result`, so no `?` can skip cleanup: every branch consumes the
/// owned [`RunCleanup`] into `settle_executed`/`settle_stopped`.
pub(in crate::campaign) trait DriveRun {
    /// Consume the driver, driving the run to a [`RunSettled`] terminal through the owned cleanup.
    fn drive(self) -> RunSettled;
}

impl DriveRun for RunDriver {
    /// Drive the run: write the manifest record, apply the unmeasured warm-up and pre-dose initial-set
    /// check, run the ten measured doses with their post-write correctness and event checks, and settle
    /// through the owned cleanup on **every** branch.
    ///
    /// Cursor order — `WritingManifest → SeedingBackground → CheckingInitialSet → Dosing`: a failure at
    /// each stage consumes that stage's state into the matching `stop_*` transition and settles the
    /// resulting [`RunExecutionStopped`](crate::campaign::run_cursor::RunExecutionStopped) through
    /// [`RunCleanup::settle_stopped`]; dose exhaustion settles the
    /// [`RunExecuted`](crate::campaign::run_cursor::RunExecuted) through [`RunCleanup::settle_executed`].
    /// The measured client is borrowed only through short `cleanup.client()` borrows that each end before
    /// the by-value settle, so cleanup is free to be consumed on every exit.
    ///
    /// Event-counter registration happens only *after* the warm-up, the subscription snapshot, and the
    /// initial-set verification (see [`ConnectedClient::register_dose_events`]), so neither snapshot nor
    /// warm-up events are counted toward any dose.
    fn drive(self) -> RunSettled {
        let RunDriver {
            cleanup,
            writing,
            context,
        } = self;

        // Cursor entry: write the run's immutable manifest record from the owned bound context. A
        // sink-write failure stops here. The context (manifest + resolved dataset) threads onward through
        // the cursor, so the run's identity has exactly one owner from entry.
        let seeding = match writing.write_manifest(context) {
            RunManifestStep::Seeding(seeding) => seeding,
            RunManifestStep::Stopped(stopped) => {
                return RunSettled::Incomplete(cleanup.settle_stopped(stopped));
            }
        };

        // Warm-up phase (`BackgroundSeed` stage): apply the unmeasured pinned background slice through the
        // normal insert reducers, derived from the run's own owned dataset. Any failure stops at this stage.
        if let Err(error) = cleanup
            .client()
            .seed(&seeding.run_dataset().dataset().background_operations())
            .context("applying the unmeasured pinned background warm-up slice")
        {
            return RunSettled::Incomplete(
                cleanup.settle_stopped(seeding.stop_background_seed(error)),
            );
        }
        let checking = seeding.seeded();

        // Initial-set-check phase (`InitialSetCheck` stage): subscribe to the measured target (its
        // snapshot carries the already-seeded warm-up), read the pre-dose baseline, and assert it equals
        // the seed-derived initial expected set. The subscribed target and expected set are derived inside
        // `check_initial_set` from the checking state's own owned bound context — the single run carrier —
        // so no parallel target is minted here. Any failure stops at this stage.
        if let Err(error) = check_initial_set(cleanup.client(), checking.run_dataset()) {
            return RunSettled::Incomplete(
                cleanup.settle_stopped(checking.stop_initial_set_check(error)),
            );
        }

        // Register the per-dose event callbacks now — after the warm-up, the subscription snapshot, and
        // the initial-set verification — so neither snapshot nor warm-up events are counted. The counter
        // is shared with the SDK callback thread through an `Arc` and outlives the dose loop. The target is
        // derived again from the still-owned checking context at this use site, never a long-lived parallel
        // variable, so it cannot drift from the subscribed and per-dose target.
        let counter = Arc::new(DoseEventCounter::new());
        cleanup
            .client()
            .register_dose_events(checking.run_dataset().subscribed_table(), &counter);

        // Consume the checking state into the dose ladder now that the pre-dose subscription, verification,
        // and callback registration have all completed against its context.
        let mut dosing = checking.checked();

        // Dose ladder: draw the next dose, run its measured batch and checks, write its observation —
        // routing an effect or write failure through the matching consuming stop transition — and settle
        // through the owned cleanup at exhaustion. Each dose's identity — its context, active dose, batch,
        // subscribed target, and expected set — is derived inside `run_one_dose` from the single borrowed
        // `RunAwaitingDose` state, so none can be supplied independently; the borrow ends before the state
        // is consumed into `write_observation`/`stop_execution`.
        let executed = loop {
            match dosing.next_dose() {
                RunDoseStep::Exhausted(executed) => break executed,
                RunDoseStep::Awaiting(awaiting) => {
                    match run_one_dose(cleanup.client(), &awaiting, &counter) {
                        Ok(evidence) => match awaiting.write_observation(evidence) {
                            RunObservationStep::Dosing(next) => dosing = next,
                            RunObservationStep::Stopped(stopped) => {
                                return RunSettled::Incomplete(cleanup.settle_stopped(stopped));
                            }
                        },
                        Err(error) => {
                            return RunSettled::Incomplete(
                                cleanup.settle_stopped(awaiting.stop_execution(error)),
                            );
                        }
                    }
                }
            }
        };
        cleanup.settle_executed(executed)
    }
}

/// The pre-dose initial-set check: subscribe to the run's target (its snapshot carries the already-seeded
/// unmeasured warm-up), read the pre-dose baseline out of the cache, and assert it equals the seed-derived
/// initial expected set. Takes the checking state's bound `context` as its sole identity-bearing argument
/// and derives the subscribed target and dataset from it, so no independently supplied target/dataset pair
/// can disagree. Kept as a fallible helper so the driver can route any failure through the
/// `InitialSetCheck` stop transition rather than a bare `?` that would strand the cleanup.
fn check_initial_set(client: &ConnectedClient, context: &RunDataset) -> Result<()> {
    let target = context.subscribed_table();
    client
        .subscribe(target)
        .context("subscribing to the measured target for the initial-set check")?;
    let observed = client.read_current(target);
    let expected = target.expected_initial(context.dataset());
    observed
        .assert_equals(&expected)
        .context("pre-dose initial result-set correctness")?;
    Ok(())
}

/// Run one measured dose and return its measured [`DoseEvidence`], or fail with the effect evidence the
/// driver routes into a `Dosing`-stage stop. Takes the borrowed [`RunAwaitingDose`] state as its sole
/// identity-bearing argument and derives everything the measured work needs from it — the bound context and
/// dataset, the active dose, this dose's batch, the subscribed target, and the cumulative expected set — so
/// none can be supplied independently or disagree. The borrow ends when this returns, leaving the awaiting
/// state free to be consumed into `write_observation` (on `Ok`) or `stop_execution` (on `Err`).
///
/// All fallible measured work is done here, strictly *before* the caller's `write_observation`, so an
/// effect failure reaches `stop_execution` while the awaiting-dose state is still held. This helper
/// produces only the measured evidence; the awaiting-dose state assembles the observation from its own
/// owned bound context, so no run coordinate, manifest reference, or record is minted here.
///
/// The sequence: apply the spec-mandated inter-dose pacing (never before the first dose), sample the
/// pre-dose result set, issue the Anton-shaped measured batch, sample the post-dose result set, then drain
/// the delivered events at the quiescent boundary the batch's confirmations established. The queried net
/// delta is computed from the two verified result sets with **checked** conversions and a **checked**
/// subtraction (never `usize as i64`) and fed with the drained counts to [`EventEvidence::checked`].
/// Finally the post-dose set is asserted equal to the derived cumulative expected set before the evidence
/// is returned.
fn run_one_dose(
    client: &ConnectedClient,
    awaiting: &RunAwaitingDose,
    counter: &Arc<DoseEventCounter>,
) -> Result<DoseEvidence> {
    let context = awaiting.run_dataset();
    let active = awaiting.active();
    let dataset = context.dataset();
    let target = context.subscribed_table();
    let batch = DoseBatch::new(dataset, active);

    // Spec-mandated experimental pacing: a fixed delay *between* doses, outside every measured interval
    // and before the pre-dose result-set sample. Applied before every dose except the first (nothing
    // precedes it), and never after the last.
    if active != DoseIndex::ALL[0] {
        sleep(Duration::from_millis(BATCH_DELAY_MS));
    }

    // The pre-dose read must sit at the same boundary as the previous dose's drain: before this dose's
    // first measured issue. The pre/post reads bracket exactly the whole-dose window the drained counts
    // cover.
    let pre = client.read_current(target);
    let latencies = client
        .measure_dose(&batch.operations())
        .with_context(|| format!("measuring dose {}", active.get()))?;
    let post = client.read_current(target);

    // Drain the dose's delivered events at the proven quiescent boundary: every measured confirmation's
    // row callbacks fired before that confirmation was received, and `measure_dose` returned only after
    // all confirmations, so no callback is in flight here.
    let counts = counter.drain();

    // Exact post-set verification comes *first*: the actual post-dose result set must equal the derived
    // cumulative expected set for this dose before any event evidence is derived. Successful event
    // evidence is therefore impossible until the measured confirmations, the quiescent drain, and this
    // exact post-set check have all succeeded.
    let expected = target.expected_through_dose(dataset, active);
    post.assert_equals(&expected)
        .with_context(|| format!("post-dose result-set correctness for dose {}", active.get()))?;

    // Only then derive the queried net delta from the verified result sets: checked `i64` conversions
    // and a checked subtraction, never a lossy `usize as i64`. Fed with the drained counts to the checked
    // event-net identity (inserts − deletes == queried net delta).
    let pre_len =
        i64::try_from(pre.len()).context("pre-dose result-set cardinality does not fit i64")?;
    let post_len =
        i64::try_from(post.len()).context("post-dose result-set cardinality does not fit i64")?;
    let net_delta = post_len
        .checked_sub(pre_len)
        .context("post-minus-pre net row delta overflowed i64")?;
    let events = EventEvidence::checked(
        counts.inserts(),
        counts.deletes(),
        counts.updates(),
        net_delta,
    )?;

    Ok(DoseEvidence::new(latencies, events))
}
