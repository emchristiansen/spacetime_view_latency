//! A connected measured-subscriber client: connect, seed, subscribe, read back.

use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{anyhow, ensure, Context, Result};
use spacetimedb_sdk::__codegen::InternalError;
use spacetimedb_sdk::{DbContext, Identity};

use crate::dataset::seed_op::SeedOp;
use crate::dataset::seeded_visibility::SeededVisibility;
use crate::dataset::subscribed_rows::SubscribedRows;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::module_artifact::bindings::{
    insert_chronicle_message, insert_message, insert_message_visibility, DbConnection,
    ReducerEventContext,
};
use crate::observation::confirmation_set::ConfirmationSet;
use crate::observation::dose_latency_accumulator::DoseLatencyAccumulator;
use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::{BATCH_SIZE, BATCH_SIZE_USIZE, CONFIRMED_READS, ROW_PAYLOAD};

/// Wait budget for the initial connection handshake (`on_connect` / `on_connect_error`).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Wait budget for one confirmed seeding reducer round trip. Generous relative to a single
/// fsync-confirmed insert; seeding is not the measured operation.
const REDUCER_TIMEOUT: Duration = Duration::from_secs(60);
/// Wait budget for the initial subscription snapshot to be applied.
const SUBSCRIPTION_TIMEOUT: Duration = Duration::from_secs(30);
/// Total wait budget for one measured dose batch (phase B) to be fully confirmed, mirroring Anton's
/// single 60 s per-batch completion barrier (`batch_done_rx.recv_timeout(60s)`). This is one budget
/// for the *whole* batch, not per callback: a single absolute deadline is anchored the moment issuing
/// completes (matching Anton, whose one `recv_timeout` begins after the issue loop), and every
/// receive draws its remaining time from that deadline — so all [`BATCH_SIZE`] writes must confirm
/// within this budget of the last issue, never 60 s times the callback count.
const MEASURED_BATCH_TIMEOUT: Duration = Duration::from_secs(60);
/// Total wait budget for the Chronicle prerequisite batch (phase A) to be fully confirmed — a
/// distinct barrier and deadline from [`MEASURED_BATCH_TIMEOUT`], so prerequisite confirmation timing
/// can never overlap the measured batch. Generous like [`REDUCER_TIMEOUT`]: the prerequisites are
/// unmeasured seeding-shaped writes, and phase A must fully complete before any measured write issues.
const PREREQUISITE_BATCH_TIMEOUT: Duration = Duration::from_secs(60);

/// The flattened outcome of an insertion reducer as delivered to its completion callback.
type ReducerOutcome = std::result::Result<std::result::Result<(), String>, InternalError>;
/// A boxed reducer completion callback. `Box<dyn FnOnce + Send + 'static>` itself satisfies
/// the generated `_then` methods' `impl FnOnce(…) + Send + 'static` bound, so the
/// channel-signalling callback can be built once and passed uniformly for every reducer.
type BoxedReducerCallback = Box<dyn FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static>;

/// One callback outcome from the Chronicle **prerequisite** batch (phase A), delivered from the SDK
/// callback thread to the prerequisite barrier over an [`mpsc`] channel. It carries no timing — a
/// prerequisite `chronicle_message` insert is unmeasured; the barrier only requires that every one
/// confirms *successfully* before any measured write is issued, so the phase and its message type are
/// kept distinct from the measured batch and can never overlap it.
enum PrerequisiteMessage {
    /// A prerequisite write confirmed successfully — presence only, addressed by issue `index`.
    Confirmed { index: usize },
    /// A prerequisite callback reported a failure (a reducer-returned error or an SDK internal error).
    Failed { index: usize, error: String },
}

/// One callback outcome from the **measured** dose batch (phase B), delivered from the SDK callback
/// thread to the measured barrier over an [`mpsc`] channel. The `index` slot-addresses the callback
/// within the batch, so the barrier can reject a duplicate or out-of-range fire and seal the samples
/// in issue order — never relying on callback *delivery* order.
enum MeasuredMessage {
    /// A measured write's confirmation, carrying the round-trip latency captured in its callback.
    Confirmed { index: usize, sample: LatencySample },
    /// A measured callback reported a failure (a reducer-returned error or an SDK internal error).
    Failed { index: usize, error: String },
}

/// A live measured-subscriber client bound to one provisioned database.
///
/// Connecting captures the **server-issued** connection identity (the measured role's
/// authenticated identity, per Option A) from `on_connect`. Seeding drives the module's
/// insertion reducers one confirmed round trip at a time. Subscription happens *after*
/// seeding so the initial snapshot is the deterministic baseline, which is then read out of
/// the client cache for correctness checks — no reliance on incremental-update ordering.
pub(crate) struct ConnectedClient {
    conn: DbConnection,
    handle: JoinHandle<()>,
    measured_identity: Identity,
}

impl ConnectedClient {
    /// Connect to the database `database_identity` (canonical hex) at `server_url`, capturing
    /// the server-issued measured identity. Fails fast if the handshake errors or times out.
    pub(crate) fn connect(server_url: &str, database_identity: &str) -> Result<Self> {
        let (connect_tx, connect_rx) = mpsc::channel::<std::result::Result<Identity, String>>();
        let conn = DbConnection::builder()
            .with_uri(server_url)
            .with_database_name(database_identity)
            .with_confirmed_reads(CONFIRMED_READS)
            .on_connect({
                let connect_tx = connect_tx.clone();
                move |_ctx, identity, _token| {
                    deliver(&connect_tx, Ok(identity));
                }
            })
            .on_connect_error(move |_ctx, err| {
                deliver(&connect_tx, Err(format!("{err:?}")));
            })
            .build()
            .map_err(|e| anyhow!("building connection to {server_url}: {e:?}"))?;

        let handle = conn.run_threaded();

        let measured_identity = connect_rx
            .recv_timeout(CONNECT_TIMEOUT)
            .context("waiting for the connection handshake")?
            .map_err(|msg| anyhow!("connection handshake failed: {msg}"))?;

        Ok(Self {
            conn,
            handle,
            measured_identity,
        })
    }

    /// The server-issued measured identity captured at connect time.
    pub(crate) fn measured_identity(&self) -> Identity {
        self.measured_identity
    }

    /// Apply every seeding write in order, blocking on each reducer's confirmed completion
    /// before issuing the next. A Chronicle pair inserts the `chronicle_message` before its
    /// `message_visibility`, so each visibility row always has its chronicle target present.
    pub(crate) fn seed(&self, operations: &[SeedOp]) -> Result<()> {
        for operation in operations {
            match *operation {
                SeedOp::Message { id, sender } => {
                    self.await_reducer(|cb| {
                        self.conn.reducers.insert_message_then(
                            id,
                            sender,
                            ROW_PAYLOAD.to_string(),
                            cb,
                        )
                    })
                    .with_context(|| format!("seeding message id={id}"))?;
                }
                SeedOp::ChroniclePair { key, viewer } => {
                    self.await_reducer(|cb| {
                        self.conn.reducers.insert_chronicle_message_then(
                            key,
                            ROW_PAYLOAD.to_string(),
                            cb,
                        )
                    })
                    .with_context(|| format!("seeding chronicle_message uuid={key}"))?;
                    self.await_reducer(|cb| {
                        self.conn
                            .reducers
                            .insert_message_visibility_then(key, viewer, key, cb)
                    })
                    .with_context(|| format!("seeding message_visibility id={key}"))?;
                }
            }
        }
        Ok(())
    }

    /// Subscribe to `target` and, once the initial snapshot is applied, read its rows out of
    /// the client cache. Fails fast on a subscription error or an applied-timeout.
    pub(crate) fn subscribe_and_read(&self, target: SubscribedTable) -> Result<SubscribedRows> {
        self.subscribe_and_await_applied(target.subscription_sql())?;
        Ok(target.read_back(&self.conn))
    }

    /// Subscribe to the full `message_visibility` table, wait for its snapshot, and read every
    /// live row back for the `(viewer, message_uuid)` pair-uniqueness check. Both the measured
    /// and the growth slices' visibility rows are materialized, so the check covers the whole
    /// seeded visibility set rather than the arm view's measured-only slice.
    pub(crate) fn read_back_visibility(&self) -> Result<SeededVisibility> {
        self.subscribe_and_await_applied(SeededVisibility::subscription_sql())?;
        Ok(SeededVisibility::read_back(&self.conn))
    }

    /// Measure one cumulative dose as an Anton-shaped back-to-back measured batch, returning the
    /// issue-ordered [`RawLatencies`].
    ///
    /// The `operations` are a single dose's homogeneous insert intents — either all
    /// [`SeedOp::Message`] or all [`SeedOp::ChroniclePair`], never a mix. That is proven from the
    /// batch length plus the Chronicle-pair count: with exactly [`BATCH_SIZE`] operations a count of
    /// `0` means every op is a message and a count of [`BATCH_SIZE`] means every op is a Chronicle
    /// pair; any other count is a non-homogeneous dose and fails loud.
    ///
    /// A Chronicle dose is measured in **two ordered phases** so the measured batch is genuinely
    /// back-to-back and never depends on server-side execution ordering:
    ///
    /// - **Phase A ([`Self::confirm_prerequisites`])** issues all [`BATCH_SIZE`] `chronicle_message`
    ///   prerequisite inserts back-to-back and barriers until *every* one has confirmed successfully.
    ///   A confirmed callback establishes that the prerequisite transaction completed successfully, so
    ///   once phase A returns every prerequisite transaction has finished and a later transaction can
    ///   observe its chronicle row — no reliance on the relative execution order of two reducers on
    ///   the connection. This phase is entirely outside every measured interval.
    /// - **Phase B ([`Self::measure_writes`])** then issues all [`BATCH_SIZE`] measured writes
    ///   back-to-back — for Chronicle the `message_visibility` inserts (the writes that can refresh
    ///   subscriptions), for the message family the `message` inserts — barriers on their confirmed
    ///   round trips, and seals. With phase A already complete, no prerequisite issue is interleaved
    ///   between two measured writes, so the measured batch is uninterrupted.
    ///
    /// The message family has no prerequisites and runs phase B directly. Each measured write's
    /// `Instant` is captured immediately before it is issued and its elapsed is read only inside the
    /// confirmation callback, exactly as Anton's baseline does.
    pub(crate) fn measure_dose(&self, operations: &[SeedOp]) -> Result<RawLatencies> {
        ensure!(
            operations.len() == BATCH_SIZE_USIZE,
            "a measured dose must issue exactly {BATCH_SIZE} writes, got {}",
            operations.len()
        );
        // A homogeneous message dose has zero Chronicle pairs; a homogeneous Chronicle dose has one
        // per op. Given the length invariant above, only `0` or `BATCH_SIZE` are homogeneous — any
        // value between is a mixed dose and is rejected.
        let chronicle_pairs = operations
            .iter()
            .filter(|op| matches!(op, SeedOp::ChroniclePair { .. }))
            .count();
        ensure!(
            chronicle_pairs == 0 || chronicle_pairs == BATCH_SIZE_USIZE,
            "a measured dose must be homogeneous (all message or all chronicle-pair writes); \
             got {chronicle_pairs} chronicle pairs in a {BATCH_SIZE}-write dose"
        );

        // Phase A: for a Chronicle dose, land and confirm every prerequisite chronicle row before any
        // measured write is issued. The message family skips this phase entirely.
        if chronicle_pairs == BATCH_SIZE_USIZE {
            self.confirm_prerequisites(operations)?;
        }

        // Phase B: the Anton-shaped back-to-back measured batch (both families).
        self.measure_writes(operations)
    }

    /// Phase A of a Chronicle dose: issue every op's `chronicle_message` prerequisite insert
    /// back-to-back, then barrier until all [`BATCH_SIZE`] have confirmed successfully.
    ///
    /// Called only for the all-Chronicle family, so every op contributes exactly one prerequisite at
    /// its issue index. The barrier ([`collect_prerequisite_batch`]) requires the full expected set
    /// under its own [`PREREQUISITE_BATCH_TIMEOUT`] deadline and fails loud on any prerequisite
    /// failure, duplicate, or out-of-range fire; a confirmed callback means that prerequisite's
    /// transaction completed successfully, so a later transaction can observe the row. This phase
    /// completes before [`Self::measure_writes`] issues anything, so it is entirely outside every
    /// measured interval and cannot overlap the measured batch.
    fn confirm_prerequisites(&self, operations: &[SeedOp]) -> Result<()> {
        let (tx, rx) = mpsc::channel::<PrerequisiteMessage>();

        for (index, operation) in operations.iter().enumerate() {
            if let SeedOp::ChroniclePair { key, .. } = *operation {
                let payload = ROW_PAYLOAD.to_string();
                let prerequisite_tx = tx.clone();
                self.conn
                    .reducers
                    .insert_chronicle_message_then(
                        key,
                        payload,
                        prerequisite_callback(index, prerequisite_tx),
                    )
                    .map_err(|e| {
                        anyhow!("issuing prerequisite chronicle_message write index {index}: {e:?}")
                    })?;
            }
        }

        // Anchor the phase-A deadline now that issuing is done. `tx` outlives the barrier, so the
        // channel cannot disconnect mid-phase and every stall surfaces as a timeout.
        let deadline = Instant::now() + PREREQUISITE_BATCH_TIMEOUT;
        collect_prerequisite_batch(rx, operations.len(), deadline)
    }

    /// Phase B: issue every op's measured write back-to-back, then barrier on the confirmed round
    /// trips and seal an issue-ordered [`RawLatencies`].
    ///
    /// For the message family the measured write is the `message` insert; for a Chronicle dose it is
    /// the `message_visibility` insert (whose prerequisite chronicle row was already confirmed in
    /// phase A). No prerequisite issue is interleaved here, so the measured writes are genuinely
    /// back-to-back. The measured interval is kept free of harness-side allocation: every owned
    /// reducer argument (the message payload) and the callback's `Sender` clone are prebuilt *before*
    /// `Instant::now()`, so only the timestamp capture and the generated issue call fall inside it.
    /// The one heap allocation that remains within the interval is the SDK's own `Box::new(callback)`
    /// made inside the generated invocation — unavoidable for any caller and the identical surface
    /// Anton measures.
    fn measure_writes(&self, operations: &[SeedOp]) -> Result<RawLatencies> {
        let (tx, rx) = mpsc::channel::<MeasuredMessage>();

        for (index, operation) in operations.iter().enumerate() {
            match *operation {
                SeedOp::Message { id, sender } => {
                    let payload = ROW_PAYLOAD.to_string();
                    let measured_tx = tx.clone();
                    let start = Instant::now();
                    self.conn
                        .reducers
                        .insert_message_then(
                            id,
                            sender,
                            payload,
                            measured_callback(index, start, measured_tx),
                        )
                        .map_err(|e| {
                            anyhow!("issuing measured message write index {index}: {e:?}")
                        })?;
                }
                SeedOp::ChroniclePair { key, viewer } => {
                    // The visibility insert carries no `String` payload; prebuild only its sender
                    // clone before timing.
                    let measured_tx = tx.clone();
                    let start = Instant::now();
                    self.conn
                        .reducers
                        .insert_message_visibility_then(
                            key,
                            viewer,
                            key,
                            measured_callback(index, start, measured_tx),
                        )
                        .map_err(|e| {
                            anyhow!(
                                "issuing measured message_visibility write index {index}: {e:?}"
                            )
                        })?;
                }
            }
        }

        // Anchor the one whole-batch deadline now that issuing is done, matching Anton's single
        // post-issue `recv_timeout` rather than a fresh budget per received callback. `tx` outlives
        // the barrier, so the channel cannot disconnect mid-batch and every stall is a timeout.
        let deadline = Instant::now() + MEASURED_BATCH_TIMEOUT;
        collect_measured_batch(rx, deadline)
    }

    /// Subscribe to `sql` and block until the initial snapshot is applied, failing fast on a
    /// subscription error or an applied-timeout. Shared by the arm/control result read-back
    /// and the `message_visibility` pair-uniqueness read-back.
    fn subscribe_and_await_applied(&self, sql: String) -> Result<()> {
        let (applied_tx, applied_rx) = mpsc::channel::<std::result::Result<(), String>>();
        self.conn
            .subscription_builder()
            .on_applied({
                let applied_tx = applied_tx.clone();
                move |_ctx| {
                    deliver(&applied_tx, Ok(()));
                }
            })
            .on_error(move |_ctx, err| {
                deliver(&applied_tx, Err(format!("{err:?}")));
            })
            .subscribe([sql]);

        applied_rx
            .recv_timeout(SUBSCRIPTION_TIMEOUT)
            .context("waiting for the subscription to be applied")?
            .map_err(|msg| anyhow!("subscription failed: {msg}"))
    }

    /// Disconnect and join the message-processing thread.
    pub(crate) fn disconnect(self) -> Result<()> {
        self.conn
            .disconnect()
            .map_err(|e| anyhow!("disconnecting client: {e:?}"))?;
        self.handle
            .join()
            .map_err(|_| anyhow!("client message-processing thread panicked"))?;
        Ok(())
    }

    /// Issue one reducer via `issue` and block until its confirmed completion callback fires,
    /// translating a reducer-returned error or an internal error into a failure.
    fn await_reducer(
        &self,
        issue: impl FnOnce(BoxedReducerCallback) -> spacetimedb_sdk::Result<()>,
    ) -> Result<()> {
        let (done_tx, done_rx) = mpsc::channel::<std::result::Result<(), String>>();
        let callback: BoxedReducerCallback = Box::new(move |_ctx, outcome| {
            let flattened = match outcome {
                Ok(Ok(())) => Ok(()),
                Ok(Err(msg)) => Err(format!("reducer returned an error: {msg}")),
                Err(internal) => Err(format!("internal error awaiting reducer: {internal:?}")),
            };
            deliver(&done_tx, flattened);
        });
        issue(callback).map_err(|e| anyhow!("issuing reducer: {e:?}"))?;
        done_rx
            .recv_timeout(REDUCER_TIMEOUT)
            .context("waiting for confirmed reducer completion")?
            .map_err(|msg| anyhow!("{msg}"))
    }
}

/// Phase A barrier: receive the prerequisite batch's callbacks into a slot-indexed
/// [`ConfirmationSet`] and return once every `expected` prerequisite has confirmed successfully.
///
/// Factored out of [`ConnectedClient::confirm_prerequisites`] so it depends on nothing but the
/// channel, the expected count, and one absolute `deadline` — no live server. A duplicate or
/// out-of-range fire (rejected by the set) or any [`PrerequisiteMessage::Failed`] fails loud; every
/// receive draws its remaining budget from the single `deadline`, so the whole phase shares one total
/// timeout. Delivery order is never assumed — the set tracks presence, not order.
fn collect_prerequisite_batch(
    rx: mpsc::Receiver<PrerequisiteMessage>,
    expected: usize,
    deadline: Instant,
) -> Result<()> {
    let mut prerequisites = ConfirmationSet::new(expected);

    while !prerequisites.is_complete() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let message = rx
            .recv_timeout(remaining)
            .context("waiting for a confirmed prerequisite callback")?;
        match message {
            PrerequisiteMessage::Confirmed { index } => {
                prerequisites.record(index)?;
            }
            PrerequisiteMessage::Failed { index, error } => {
                return Err(anyhow!(
                    "prerequisite chronicle_message write index {index} failed: {error}"
                ));
            }
        }
    }

    Ok(())
}

/// Phase B barrier: receive the measured batch's callbacks into a slot-indexed
/// [`DoseLatencyAccumulator`] and seal the samples in issue order once every measured write confirms.
///
/// The load-bearing measured barrier, factored out of [`ConnectedClient::measure_writes`] so it
/// depends on nothing but the channel and one absolute `deadline` — no live server. A duplicate or
/// out-of-range fire (rejected by the accumulator) or any [`MeasuredMessage::Failed`] fails loud;
/// delivery order is never assumed, and the seal is in issue order `0..BATCH_SIZE`. Every receive
/// draws its remaining budget from the single `deadline`, so the whole batch shares one total timeout.
fn collect_measured_batch(
    rx: mpsc::Receiver<MeasuredMessage>,
    deadline: Instant,
) -> Result<RawLatencies> {
    let mut measured = DoseLatencyAccumulator::new();

    while !measured.is_complete() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let message = rx
            .recv_timeout(remaining)
            .context("waiting for a confirmed measured-batch callback")?;
        match message {
            MeasuredMessage::Confirmed { index, sample } => {
                measured.record(index, sample)?;
            }
            MeasuredMessage::Failed { index, error } => {
                return Err(anyhow!("measured write index {index} failed: {error}"));
            }
        }
    }

    measured.seal()
}

/// Deliver a one-shot `outcome` to the waiting harness thread over `sender`.
///
/// Every one of these channels is awaited with `recv_timeout` on the harness thread while the
/// SDK drives the sending callback on its own background thread. That ordering has exactly one
/// failure mode: the harness already timed out and dropped the receiver before this late
/// callback fired. [`mpsc::SendError`] has that single cause, so the failure is *modeled* here
/// as the expected late-callback race — there is no result left to deliver and nothing to fail.
/// It is deliberately neither discarded with a bare `let _` (the failure has a specific,
/// reasoned meaning) nor propagated (a callback cannot return an error) nor panicked (which
/// would abort the SDK background thread and lose the run's real error).
fn deliver<T>(sender: &mpsc::Sender<T>, outcome: T) {
    match sender.send(outcome) {
        Ok(()) => {}
        // The receiver has already stopped waiting (recv timed out and dropped it); the value
        // returned inside the error is the signal we no longer have anyone to hand it to.
        Err(mpsc::SendError(_unreceived)) => {}
    }
}

/// Build the completion callback for one measured write at issue `index`, capturing `start` so the
/// round-trip elapsed is read only when the confirmation fires — the measured interval is exactly
/// issue-to-confirm, matching Anton's `start.elapsed()` inside the callback. A reducer-returned or
/// internal error becomes a [`MeasuredMessage::Failed`] rather than a sample.
///
/// Returns the closure **unboxed** as `impl FnOnce`, not a [`BoxedReducerCallback`]: the generated
/// `insert_*_then` methods accept `impl FnOnce` and box it themselves inside
/// `invoke_reducer_with_callback` (`Box::new(callback)`), so the only heap allocation is the SDK's own
/// unavoidable one, made *inside* the generated invocation — the same surface Anton measures. Building
/// this closure is a pure stack construction moving `index`/`start` (both `Copy`) and the already
/// pre-cloned `tx`, so nothing this harness contributes allocates between `start` and the issue call.
fn measured_callback(
    index: usize,
    start: Instant,
    tx: mpsc::Sender<MeasuredMessage>,
) -> impl FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static {
    move |_ctx, outcome| {
        let message = match outcome {
            Ok(Ok(())) => MeasuredMessage::Confirmed {
                index,
                sample: LatencySample::from_elapsed(start.elapsed()),
            },
            Ok(Err(msg)) => MeasuredMessage::Failed {
                index,
                error: format!("reducer returned an error: {msg}"),
            },
            Err(internal) => MeasuredMessage::Failed {
                index,
                error: format!("internal error awaiting reducer: {internal:?}"),
            },
        };
        deliver(&tx, message);
    }
}

/// Build the completion callback for one Chronicle prerequisite write at issue `index`. It carries no
/// timing — the prerequisite is unmeasured — only its confirmed presence, so the phase-A barrier can
/// require every prerequisite to confirm and never drop a prerequisite failure as unread.
///
/// Returned **unboxed** for the same reason as [`measured_callback`]: the generated method boxes it
/// internally, so no caller-side heap allocation is added.
fn prerequisite_callback(
    index: usize,
    tx: mpsc::Sender<PrerequisiteMessage>,
) -> impl FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static {
    move |_ctx, outcome| {
        let message = match outcome {
            Ok(Ok(())) => PrerequisiteMessage::Confirmed { index },
            Ok(Err(msg)) => PrerequisiteMessage::Failed {
                index,
                error: format!("reducer returned an error: {msg}"),
            },
            Err(internal) => PrerequisiteMessage::Failed {
                index,
                error: format!("internal error awaiting reducer: {internal:?}"),
            },
        };
        deliver(&tx, message);
    }
}

#[cfg(test)]
mod tests;
