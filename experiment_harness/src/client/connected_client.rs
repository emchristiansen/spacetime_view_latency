//! A connected measured-subscriber client: connect, seed, subscribe, read back.

use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{anyhow, ensure, Context, Error, Result};
use spacetimedb_sdk::__codegen::InternalError;
use spacetimedb_sdk::{DbContext, Identity, Table, TableWithPrimaryKey, Timestamp};

use crate::client::measured_step_failure::MeasuredStepFailure;
use crate::client::paced_append::PacedAppend;
use crate::client::paced_append_failure::PacedAppendFailure;
use crate::client::reconnect_failure::ReconnectFailure;
use crate::client::view_event_shape::ViewEventShape;
use crate::dataset::seed_op::SeedOp;
use crate::dataset::seeded_visibility::SeededVisibility;
use crate::dataset::subscribed_rows::SubscribedRows;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::entity_owner_pilot::pilot_params::PILOT_ROW_PAYLOAD;
use crate::module_artifact::bindings::{
    insert_chronicle_message, insert_control_activity, insert_entity_owner,
    insert_indexed_control_activity, insert_message, insert_message_visibility,
    record_control_activity, record_first_control_activity, update_entity_owner, ControlActivity,
    ControlActivityEmptyViewTableAccess, ControlActivityLatestByControlViewTableAccess,
    ControlActivitySenderViewTableAccess, ControlActivityTableAccess, ControlRegistry,
    ControlRegistryAllViewTableAccess, ControlRegistryTableAccess, DbConnection, EntityOwner,
    EntityOwnerSenderViewTableAccess, EntityOwnerTableAccess, IndexedControlActivity,
    IndexedControlActivitySenderViewTableAccess, IndexedControlActivityTableAccess,
    ReducerEventContext, SubscriptionHandle,
};
use crate::observation::confirmation_set::ConfirmationSet;
use crate::observation::dose_event_counter::DoseEventCounter;
use crate::observation::dose_latency_accumulator::DoseLatencyAccumulator;
use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::{BATCH_SIZE, BATCH_SIZE_USIZE, CONFIRMED_READS, ROW_PAYLOAD};
use crate::provision::teardown::into_error;
use crate::view_read_set_campaign::campaign_params::{
    CHANNEL_SAMPLE_COUNT_USIZE, PACED_SAMPLE_DELAY_MS,
};
use crate::view_read_set_campaign::measured_target::MeasuredTarget;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;
use crate::view_read_set_campaign::saturated_timing_accumulator::SaturatedTimingAccumulator;
use crate::view_read_set_campaign::saturated_timing_batch::SaturatedTimingBatch;
use crate::view_read_set_campaign::saturated_write_timing::SaturatedWriteTiming;
use crate::view_read_set_campaign::subscriber_delivery::delivery_counters::DeliveryCounters;

/// The `entity_owner_sender_view` subscription query name — a cross-component contract with the
/// module's `#[view(accessor = entity_owner_sender_view, …)]`, so it is a named constant rather
/// than an inline literal.
///
/// Reached by
/// [`MeasuredTarget::subscription_sql`](crate::view_read_set_campaign::measured_target::MeasuredTarget::subscription_sql)
/// as well as by this module's own helpers, so the fresh-server campaign names the same table string
/// this client does rather than declaring a second copy that could drift from it.
pub(crate) const TABLE_ENTITY_OWNER_SENDER_VIEW: &str = "entity_owner_sender_view";
/// The `entity_owner` base-table subscription query name — the same kind of cross-component
/// contract, with the module's `#[table(accessor = entity_owner, public)]`, shared for the same
/// reason.
pub(crate) const TABLE_ENTITY_OWNER: &str = "entity_owner";
/// The `control_activity_sender_view` subscription query name — the same cross-component contract
/// with the module's `#[view(accessor = control_activity_sender_view, …)]`. The empty-view
/// reproducer subscribes to this one as its composition control: it proves the seeded rows are
/// present and visible to the caller, so an empty answer from the view below is emptiness by
/// predicate rather than an empty table.
pub(crate) const TABLE_CONTROL_ACTIVITY_SENDER_VIEW: &str = "control_activity_sender_view";
/// The `indexed_control_activity_sender_view` subscription query name — the contract with the
/// module's `#[view(accessor = indexed_control_activity_sender_view, …)]`, site 4's indexed sender
/// arm and the relation the append-only E2 calibration pilot times.
pub(crate) const TABLE_INDEXED_CONTROL_ACTIVITY_SENDER_VIEW: &str =
    "indexed_control_activity_sender_view";
/// The `indexed_control_activity` base-table subscription query name — the contract with the
/// module's `#[table(accessor = indexed_control_activity, public)]`. The calibration pilot
/// subscribes to it untimed after its batch, as the composition witness proving the unrelated
/// population it measured against was actually seeded.
pub(crate) const TABLE_INDEXED_CONTROL_ACTIVITY: &str = "indexed_control_activity";
/// The `control_activity_empty_view` subscription query name — the contract with the module's
/// `#[view(accessor = control_activity_empty_view, …)]`, the typed-contradiction capability
/// reproducer for site 4's admin empty-result question.
pub(crate) const TABLE_CONTROL_ACTIVITY_EMPTY_VIEW: &str = "control_activity_empty_view";
/// The `control_activity_latest_by_control_view` subscription query name — the contract with the
/// module's `#[view(accessor = control_activity_latest_by_control_view, …)]`, the bounded
/// capability probe for site 4's discovery comparator.
pub(crate) const TABLE_CONTROL_ACTIVITY_LATEST_BY_CONTROL_VIEW: &str =
    "control_activity_latest_by_control_view";
/// The `control_activity` base-table subscription query name — the contract with the module's
/// `#[table(accessor = control_activity, public)]`. The registry reproducer subscribes to it as the
/// audit-history composition control: Arm B's latest-per-control answer means nothing unless the N
/// history rows it summarizes are known present.
pub(crate) const TABLE_CONTROL_ACTIVITY: &str = "control_activity";
/// The `control_registry_all_view` subscription query name — the contract with the module's
/// `#[view(accessor = control_registry_all_view, …)]`, site 4's discovery Arm A.
pub(crate) const TABLE_CONTROL_REGISTRY_ALL_VIEW: &str = "control_registry_all_view";
/// The `control_registry` base-table subscription query name — Arm A's matched direct
/// public-table Control, the same role base `entity_owner` plays for the sender view.
pub(crate) const TABLE_CONTROL_REGISTRY: &str = "control_registry";

/// Wait budget for the initial connection handshake (`on_connect` / `on_connect_error`).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Wait budget for one confirmed seeding reducer round trip. Generous relative to a single
/// fsync-confirmed insert; seeding is not the measured operation.
const REDUCER_TIMEOUT: Duration = Duration::from_secs(60);
/// Context for a wait on a reducer completion that produced no callback at all, added to whichever
/// [`mpsc::RecvTimeoutError`] ended it. Named once because the two ends classify differently and
/// must still read as the same wait.
const AWAITING_REDUCER: &str = "waiting for confirmed reducer completion";
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
/// Wait budget for **one** paced visible-apply sample: from its measured write's issue until that
/// row's update appears in the subscriber cache.
///
/// Deliberately per-sample rather than per-batch, unlike every other budget above. The paced channel
/// holds one outstanding write at a time and waits [`PACED_SAMPLE_DELAY_MS`] between *completed*
/// samples, so a whole-batch budget would have to carry a hundred seconds of preregistered pacing and
/// would no longer bound anything about a sample. Matched to the other single-round-trip budgets: a
/// visible apply is one confirmed round trip plus its delivery.
const PACED_SAMPLE_TIMEOUT: Duration = Duration::from_secs(60);

/// The flattened outcome of an insertion reducer as delivered to its completion callback.
type ReducerOutcome = std::result::Result<std::result::Result<(), String>, InternalError>;
/// A boxed reducer completion callback. `Box<dyn FnOnce + Send + 'static>` itself satisfies
/// the generated `_then` methods' `impl FnOnce(…) + Send + 'static` bound, so the
/// channel-signalling callback can be built once and passed uniformly for every reducer.
type BoxedReducerCallback = Box<dyn FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static>;

/// One awaited reducer's completion, delivered from the SDK callback thread to the awaiting harness
/// thread over an [`mpsc`] channel.
///
/// The two failures travel apart rather than pre-flattened to one string, because the callback is
/// the only place they can be told apart at all, and the campaign's seeding step must record which
/// happened. Neither is a transport fault: the module refused the write, or the SDK failed to run it
/// on this connection's behalf. Each carries the cause's own text unprefixed, so the wording a
/// caller reads is composed in one place.
enum ReducerCompletion {
    /// The reducer ran and returned successfully.
    Confirmed,
    /// The reducer returned an error — the module's own message.
    ReducerFailed(String),
    /// The SDK reported an internal error for this call — its `Debug` rendering, captured in the
    /// callback because [`InternalError`] cannot be carried further.
    Internal(String),
}

/// One callback outcome from the Chronicle **prerequisite** batch (phase A), delivered from the SDK
/// callback thread to the prerequisite barrier over an [`mpsc`] channel. It carries no timing — a
/// prerequisite `message_visibility` insert is unmeasured; the barrier only requires that every one
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

/// One event the campaign's **paced visible-apply** channel (E2) waits on, over one [`mpsc`] channel
/// shared by the whole batch.
///
/// Two producers, only one of which speaks on the success path: the batch's retained observer sends
/// [`Self::Visible`], and each write's reducer callback sends [`Self::Failed`] and nothing else. A
/// sample stops on visibility, not confirmation, so a success message would be unread traffic on the
/// measured path.
enum PacedMessage {
    /// A subscribed row was updated in the client cache, identified by its primary key and carrying
    /// the instant the observer saw it.
    ///
    /// The instant travels in the message because it is the sample's *endpoint*: the observer runs
    /// on the SDK's callback thread and one observer serves the whole batch, so it cannot subtract
    /// the current sample's start itself, and reading a clock after the barrier wakes would put this
    /// thread's scheduling inside every sample.
    Visible {
        entity_uuid: u64,
        observed_at: Instant,
    },
    /// A paced write failed at the reducer, so its visibility will never arrive.
    Failed { index: usize, error: String },
}

/// One event the **paced append** channel waits on, over one [`mpsc`] channel shared by the whole
/// batch.
///
/// The insert-shaped twin of [`PacedMessage`], kept separate rather than widened, because the two
/// channels stop on different events and match on different keys: an update to one of a fixed set of
/// cycled keys there, the arrival of a brand-new row here. One enum serving both would make the
/// key's meaning depend on which caller built the channel.
enum PacedAppendMessage {
    /// A subscribed row was inserted into the client cache, identified by its primary key and
    /// carrying the instant the observer saw it.
    ///
    /// The instant travels in the message for the same reason it does on the update channel: the
    /// observer runs on the SDK's callback thread and one observer serves the whole batch, so
    /// reading a clock after the barrier wakes would put this thread's scheduling inside every
    /// sample.
    Visible { id: u64, observed_at: Instant },
    /// A paced append failed at the reducer, so *that* append's visibility will never arrive.
    ///
    /// **Keyed by `id` as well as `index` — defensive correlation, not a reachable race.** Under the
    /// pinned SDK an append yields a visibility *or* a failure, never both: a successful
    /// `ReducerResult` applies the update and its row callbacks and then completes the reducer
    /// callback with success, which this channel deliberately reports as nothing at all, while a
    /// reducer error or internal error completes the callback *without* applying any update
    /// (`db_connection.rs:195-221,223-254`). `pop_call_info` removes the callback, so it fires once.
    /// A parse failure ends the connection rather than producing a second contradictory completion.
    ///
    /// So no interleaving in this pin delivers a visibility and then a contradictory failure for one
    /// append. The key is kept anyway because it costs one `u64` and makes the barrier's rule total:
    /// a failure ends the sample it names rather than whichever sample happens to be waiting, so an
    /// unexpected protocol state cannot be read as the current sample's outcome. `index` rides along
    /// for the diagnostic; `id` is what decides.
    Failed {
        index: usize,
        id: u64,
        error: String,
    },
}

/// One callback outcome from the campaign's **saturated** batch (E1).
///
/// Carries both offsets rather than a latency, as [`SaturatedWriteTiming`] does: the estimand is a
/// per-write service time only if the writes were issued back-to-back and confirmed in order, and a
/// vector of differences cannot distinguish that from drifted issue spacing. The issue offset is
/// captured on the issuing thread, the confirmation offset in the callback, both against one origin
/// per batch.
enum SaturatedMessage {
    /// A saturated write's confirmation, with its issue and confirmation offsets from the batch's
    /// common origin.
    Confirmed {
        index: usize,
        issue_offset_nanos: u128,
        confirmation_offset_nanos: u128,
    },
    /// A saturated callback reported a failure (a reducer-returned error or an SDK internal error).
    Failed { index: usize, error: String },
}

/// What one connection attempt left behind when it did not yield a live client.
///
/// Every variant is clientless: [`release_connection`] always joins, and a join returns only once
/// the thread has ended (`Err` meaning it ended by panicking). The variants distinguish *how*,
/// because the campaign treats abnormal cleanup as terminal even though nothing leaked. `build`
/// returns before `run_threaded`, so the first two are genuinely different post-conditions rather
/// than one restated.
enum ConnectFailure {
    /// Never built: nothing was acquired and nothing needed releasing.
    NotBuilt(MeasuredStepFailure),
    /// Built and threaded, then failed; the thread was released cleanly.
    ReleasedAfterStart(MeasuredStepFailure),
    /// Built and threaded, then failed, and the thread ended abnormally — a disconnect the SDK
    /// refused, or a panicking join. No client remains, but the cleanup is itself a terminal release
    /// failure. Carries cause and release error aggregated, neither hiding the other.
    ReleasedWithError(Error),
}

/// Whether a connection being released was believed live.
///
/// Decides how one SDK error is read, and the two readings are opposite. `DbConnectionImpl::disconnect`
/// has exactly one failure in the pinned source — [`spacetimedb_sdk::Error::Disconnected`], returned
/// when the connection is no longer active — which is a fault for a caller that believed it live and
/// the expected state for one cleaning up after a handshake that never completed.
enum ReleaseExpectation {
    Live,
    Unhandshaken,
}

/// A live measured-subscriber client bound to one provisioned database.
///
/// Connecting captures the **server-issued** connection identity (the measured role's
/// authenticated identity, per Option A) from `on_connect`. Seeding drives the module's
/// insertion reducers one confirmed round trip at a time. Subscription happens *after*
/// seeding so the initial snapshot is the deterministic baseline, which is then read out of
/// the client cache for correctness checks — no reliance on incremental-update ordering.
///
/// **The credential is retained in memory and nowhere else.** The campaign's reconnect channel
/// authenticates as the *same* identity, so this holds the token the handshake issued and the URL to
/// dial again. The type derives nothing, carries no `Serialize`, and exposes no accessor for either,
/// so neither can reach a ledger line, a diagnostic, or a panic message.
pub(crate) struct ConnectedClient {
    conn: DbConnection,
    handle: JoinHandle<()>,
    measured_identity: Identity,
    database_name: String,
    /// The URL this connection was dialled at, retained so the reconnect dials the same server.
    server_url: String,
    /// The credential the handshake issued, retained solely so the reconnect can present it again.
    token: String,
}

impl ConnectedClient {
    /// Connect to the database `database_identity` (canonical hex) at `server_url`, capturing
    /// the server-issued measured identity. Fails fast if the handshake errors or times out.
    ///
    /// Every failure is reported only after cleanup, so an `Err` from here never leaves a running
    /// message-processing thread; where the release itself failed, its error is aggregated with the
    /// cause.
    pub(crate) fn connect(server_url: &str, database_identity: &str) -> Result<Self> {
        Self::establish(server_url, database_identity, None).map_err(|failure| match failure {
            ConnectFailure::NotBuilt(cause) | ConnectFailure::ReleasedAfterStart(cause) => {
                cause.into_error()
            }
            ConnectFailure::ReleasedWithError(error) => error,
        })
    }

    /// Open one connection, optionally presenting a `token` a previous connection was issued.
    ///
    /// Shared by the first connect and the token-preserving reconnect, so the two cannot diverge in
    /// how they build, run, or await a connection — the only difference is the token, which is what
    /// the reconnect channel's estimand turns on.
    ///
    /// **No `?` may cross `run_threaded`.** Before it, nothing has been acquired; after it, a
    /// message-processing thread exists and every exit either returns the live client or explicitly
    /// releases that thread and reports how the release went. The three [`ConnectFailure`] variants
    /// are those post-conditions.
    fn establish(
        server_url: &str,
        database_identity: &str,
        token: Option<String>,
    ) -> std::result::Result<Self, ConnectFailure> {
        let (connect_tx, connect_rx) =
            mpsc::channel::<std::result::Result<(Identity, String), String>>();
        let built = DbConnection::builder()
            .with_uri(server_url)
            .with_database_name(database_identity)
            .with_confirmed_reads(CONFIRMED_READS)
            .with_token(token)
            .on_connect({
                let connect_tx = connect_tx.clone();
                move |_ctx, identity, token| {
                    deliver(&connect_tx, Ok((identity, token.to_string())));
                }
            })
            .on_connect_error(move |_ctx, err| {
                deliver(&connect_tx, Err(format!("{err:?}")));
            })
            .build();

        let conn = match built {
            Ok(conn) => conn,
            Err(e) => {
                return Err(ConnectFailure::NotBuilt(
                    MeasuredStepFailure::Infrastructure(anyhow!(
                        "building connection to {server_url}: {e:?}"
                    )),
                ))
            }
        };

        // A message-processing thread now exists. Nothing below may `?`.
        let handle = conn.run_threaded();

        let cause = match connect_rx.recv_timeout(CONNECT_TIMEOUT) {
            Ok(Ok((measured_identity, token))) => {
                return Ok(Self {
                    conn,
                    handle,
                    measured_identity,
                    database_name: database_identity.to_string(),
                    server_url: server_url.to_string(),
                    token,
                })
            }
            Ok(Err(msg)) => {
                MeasuredStepFailure::Infrastructure(anyhow!("connection handshake failed: {msg}"))
            }
            Err(mpsc::RecvTimeoutError::Timeout) => MeasuredStepFailure::Timeout(anyhow!(
                "the connection handshake with {server_url} did not answer within \
                 {CONNECT_TIMEOUT:?}"
            )),
            // Not an elapsed bound: both senders live in callbacks the builder owns, so a
            // disconnect means the SDK dropped them without invoking either.
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                MeasuredStepFailure::Infrastructure(anyhow!(
                    "the connection handshake channel with {server_url} disconnected without \
                     reporting either an identity or an error"
                ))
            }
        };

        let release = release_connection(&conn, handle, ReleaseExpectation::Unhandshaken);
        if release.is_empty() {
            Err(ConnectFailure::ReleasedAfterStart(cause))
        } else {
            let mut errors = vec![cause.into_error()];
            errors.extend(release);
            Err(ConnectFailure::ReleasedWithError(into_error(errors)))
        }
    }

    /// The server-issued measured identity captured at connect time.
    pub(crate) fn measured_identity(&self) -> Identity {
        self.measured_identity
    }

    /// The database name this connection was opened with — the exact string handed to
    /// `with_database_name` above.
    ///
    /// Retained because the SDK uses that string verbatim as the `db` label on the per-database
    /// metrics its websocket loop maintains, so this is the only value that names *this*
    /// connection's series. Anything that meters the connection takes the label from here rather
    /// than accepting one: a wrong label does not fail, it silently selects an untouched series that
    /// reads zero at both ends of any window.
    pub(crate) fn database_name(&self) -> &str {
        &self.database_name
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

    /// Subscribe to `target` and block until its initial snapshot is applied, **without** reading. The
    /// measured driver subscribes only *after* the unmeasured warm-up slice has been seeded, so the
    /// snapshot already carries the warm-up rows. It then reads the pre-dose baseline via
    /// [`Self::read_current`] for the initial-set check and registers its dose event callbacks (via
    /// [`Self::register_dose_events`]) only *after* that check — so the snapshot, and with it the
    /// warm-up, is excluded from every dose's counts. This split (subscribe without reading) is what
    /// [`Self::subscribe_and_read`], which reads immediately, cannot express.
    pub(crate) fn subscribe(&self, target: SubscribedTable) -> Result<()> {
        self.subscribe_and_await_applied(target.subscription_sql())
    }

    /// Read `target`'s currently-subscribed rows out of the live client cache, issuing no new
    /// subscription. Lets the driver sample the result set after each measured dose confirms, to
    /// compute the queried net delta and the post-dose expected-set check.
    pub(crate) fn read_current(&self, target: SubscribedTable) -> SubscribedRows {
        target.read_back(&self.conn)
    }

    /// Register `target`'s per-dose SDK row-event callbacks against the shared [`DoseEventCounter`].
    /// Thin wrapper over [`SubscribedTable::register_events`] supplying the private connection, so the
    /// counted table cannot drift from the subscribed one. Called once, after the subscription
    /// snapshot, the initial-set verification, **and** the unmeasured warm-up (which is seeded before
    /// the subscription, so its rows arrive in the snapshot rather than as incremental events) — so
    /// neither snapshot nor warm-up events are ever counted toward a dose.
    pub(crate) fn register_dose_events(
        &self,
        target: SubscribedTable,
        counter: &Arc<DoseEventCounter>,
    ) {
        target.register_events(&self.conn, counter);
    }

    /// Subscribe to the full `message_visibility` table, wait for its snapshot, and read every
    /// live row back for the `(viewer, message_uuid)` pair-uniqueness check. Both the measured
    /// and the growth slices' visibility rows are materialized, so the check covers the whole
    /// seeded visibility set rather than the arm view's measured-only slice.
    pub(crate) fn read_back_visibility(&self) -> Result<SeededVisibility> {
        self.subscribe_and_await_applied(SeededVisibility::subscription_sql())?;
        Ok(SeededVisibility::read_back(&self.conn))
    }

    /// Seed one `entity_owner` row via its confirmed insertion reducer — the `EntityOwnerSenderView`
    /// candidate's seeding primitive. No new [`SeedOp`] variant is needed for this candidate.
    ///
    /// The unclassified wrapper over [`Self::insert_entity_owner_classified`], for the Pilot and the
    /// smoke test, whose seeding is not part of a campaign attempt. It returns that method's own
    /// cause, context included, so its `Err` is unchanged.
    pub(crate) fn insert_entity_owner(
        &self,
        entity_uuid: u64,
        owner: Identity,
        record: String,
    ) -> Result<()> {
        self.insert_entity_owner_classified(entity_uuid, owner, record)
            .map_err(MeasuredStepFailure::into_error)
    }

    /// Seed one `entity_owner` row, keeping the classified cause — the campaign's seeding primitive.
    ///
    /// Identical write and identical context to [`Self::insert_entity_owner`]; the difference is
    /// only that the cause survives. The campaign seeds inside an attempt it must classify, and a
    /// module refusal recorded as an infrastructure failure would be retried against a server that
    /// answered correctly.
    ///
    /// The context is built in the `map_err` closure, so a successful seed — every seed on the
    /// healthy path, thousands per attempt — formats nothing.
    pub(crate) fn insert_entity_owner_classified(
        &self,
        entity_uuid: u64,
        owner: Identity,
        record: String,
    ) -> std::result::Result<(), MeasuredStepFailure> {
        self.await_reducer_classified(|cb| {
            self.conn
                .reducers
                .insert_entity_owner_then(entity_uuid, owner, record, cb)
        })
        .map_err(|failure| {
            failure.context(format!("seeding entity_owner entity_uuid={entity_uuid}"))
        })
    }

    /// Apply one confirmed fixed-cardinality `entity_owner` update — the fresh-server campaign's
    /// measured mutation, issued one confirmed round trip at a time.
    ///
    /// Copies [`Self::insert_entity_owner`] exactly, over `update_entity_owner` instead of the
    /// insertion reducer, so the two writes share one confirmation primitive and cannot diverge in
    /// how they wait. The module's reducer panics when no row holds `entity_uuid`, which rolls the
    /// transaction back and arrives here as a reducer error — so an update aimed at an unseeded key
    /// fails loud at this call rather than silently inserting one and raising the cardinality the
    /// measurement holds fixed.
    ///
    /// **Takes no `owner`**, unlike its insertion counterpart, because the reducer preserves the
    /// existing row's owner. The measured write changes the payload alone, and a caller cannot ask
    /// for anything else: transferring ownership is not expressible through this method, so the
    /// measured channels cannot be pointed at a mutation with a different read-set effect.
    ///
    /// This is the *paced*, one-outstanding-write shape. The saturated channel issues its batch
    /// back-to-back without awaiting, which is a different primitive built on the measured-batch
    /// barrier, and arrives with the driver stage that needs it.
    pub(crate) fn update_entity_owner(&self, entity_uuid: u64, record: String) -> Result<()> {
        self.await_reducer(|cb| {
            self.conn
                .reducers
                .update_entity_owner_then(entity_uuid, record, cb)
        })
        .with_context(|| format!("updating entity_owner entity_uuid={entity_uuid}"))
    }

    /// Subscribe to `entity_owner_sender_view` and, once its initial snapshot is applied, read the
    /// caller-scoped rows out of the client cache — the `EntityOwnerSenderView` candidate's
    /// measurement primitive. Built directly on [`Self::subscribe_and_await_applied`], the same
    /// generic primitive [`Self::subscribe_and_read`] uses for the historical arms.
    pub(crate) fn subscribe_and_read_entity_owner_sender_view(&self) -> Result<Vec<EntityOwner>> {
        self.subscribe_entity_owner_sender_view()?;
        Ok(self.read_entity_owner_sender_view())
    }

    /// Subscribe to `entity_owner_sender_view` and block until its initial snapshot is applied,
    /// **without** reading — the Arm's subscription. The measured driver subscribes once, after the
    /// owned slice is seeded and before the ladder starts, then re-reads the cache after each rung;
    /// that split is what [`Self::subscribe_and_read_entity_owner_sender_view`] cannot express. The
    /// `subscribe`/`read_current` counterpart for this candidate.
    pub(crate) fn subscribe_entity_owner_sender_view(&self) -> Result<()> {
        self.subscribe_and_await_applied(format!("SELECT * FROM {TABLE_ENTITY_OWNER_SENDER_VIEW}"))
    }

    /// Read the caller-scoped view's currently-subscribed rows out of the live client cache,
    /// issuing no new subscription.
    pub(crate) fn read_entity_owner_sender_view(&self) -> Vec<EntityOwner> {
        self.conn.db.entity_owner_sender_view().iter().collect()
    }

    /// Seed one `control_activity` row via its confirmed insertion reducer — the site 4 empty-view
    /// reproducer's seeding primitive. Copies [`Self::insert_entity_owner`] exactly, over
    /// `insert_control_activity`, so both candidates' seeding waits on the same confirmation
    /// primitive and cannot diverge in how they wait.
    ///
    /// `ts` is an explicit parameter because the module's reducer takes one rather than reading the
    /// clock, which is what makes a seeded row reproducible from the seed alone.
    pub(crate) fn insert_control_activity(
        &self,
        id: u64,
        ts: Timestamp,
        control_uuid: u64,
        user_identity: Identity,
    ) -> Result<()> {
        self.await_reducer(|cb| {
            self.conn
                .reducers
                .insert_control_activity_then(id, ts, control_uuid, user_identity, cb)
        })
        .with_context(|| format!("seeding control_activity id={id}"))
    }

    /// Subscribe to `control_activity_sender_view` and, once its initial snapshot is applied, read
    /// the caller-scoped rows out of the client cache — the empty-view reproducer's **composition
    /// control**. Built on [`Self::subscribe_and_await_applied`], the same primitive every other
    /// candidate's subscription uses.
    pub(crate) fn subscribe_and_read_control_activity_sender_view(
        &self,
    ) -> Result<Vec<ControlActivity>> {
        self.subscribe_and_await_applied(format!(
            "SELECT * FROM {TABLE_CONTROL_ACTIVITY_SENDER_VIEW}"
        ))?;
        Ok(self.conn.db.control_activity_sender_view().iter().collect())
    }

    /// Subscribe to `control_activity_empty_view` and, once its initial snapshot is applied, read
    /// whatever reached the client cache — the reproducer's **arm**.
    ///
    /// Identical in shape to the control above, differing only in the subscribed name, so the two
    /// answers differ for one reason: the view's predicate. Reaching the read at all is itself part
    /// of the result — a query the server refused would fail in
    /// [`Self::subscribe_and_await_applied`] rather than return an empty vector, which is precisely
    /// the distinction the capability question turns on.
    pub(crate) fn subscribe_and_read_control_activity_empty_view(
        &self,
    ) -> Result<Vec<ControlActivity>> {
        self.subscribe_and_await_applied(format!(
            "SELECT * FROM {TABLE_CONTROL_ACTIVITY_EMPTY_VIEW}"
        ))?;
        Ok(self.conn.db.control_activity_empty_view().iter().collect())
    }

    /// Subscribe to `control_activity_latest_by_control_view` and block until its initial snapshot
    /// is applied, **without** reading — the latest-per-control probe's subscription.
    ///
    /// Split from the read — the [`Self::subscribe_entity_owner_sender_view`] /
    /// [`Self::read_entity_owner_sender_view`] shape rather than the empty-view reproducer's
    /// combined primitive — because this probe reads the same live subscription twice: once for the
    /// initial materialization, and again after a later row is inserted, to prove the view is
    /// actually invalidated.
    ///
    /// What P4 genuinely requires is the **cache-only** read below, which issues no new
    /// subscription: a second subscription would materialize the view afresh and so could not
    /// distinguish a maintained view from a recomputed one. The split here is the matching half of
    /// that, and follows the existing pair; it is not itself load-bearing, since a combined
    /// subscribe-and-read could have served the first read equally well.
    ///
    /// Reaching the applied callback at all is part of the result: a query the server refused fails
    /// here rather than returning an empty cache.
    pub(crate) fn subscribe_control_activity_latest_by_control_view(&self) -> Result<()> {
        self.subscribe_and_await_applied(format!(
            "SELECT * FROM {TABLE_CONTROL_ACTIVITY_LATEST_BY_CONTROL_VIEW}"
        ))
    }

    /// Read the latest-per-control view's currently-subscribed rows out of the live client cache,
    /// issuing no new subscription — the counterpart read for the subscription above.
    pub(crate) fn read_control_activity_latest_by_control_view(&self) -> Vec<ControlActivity> {
        self.conn
            .db
            .control_activity_latest_by_control_view()
            .iter()
            .collect()
    }

    /// Register all three row callbacks on the latest-per-control view, reporting each delivery's
    /// [`ViewEventShape`] over `events`.
    ///
    /// All three, because the change under test — a control's latest row replaced by a strictly
    /// later one under the same `control_uuid` key — could be reported as an in-place update or as a
    /// delete plus an insert, and the probe must record which rather than assume it. Registering
    /// only `on_update` would silently score a delete-plus-insert as no invalidation at all,
    /// inverting the probe's answer.
    ///
    /// **The callbacks are never removed**, unlike
    /// [`VisibilityObserver`](crate::view_read_set_campaign::visibility_observer::VisibilityObserver),
    /// which the paced channel must cancel before the saturated channel runs so instrumentation does
    /// not charge a channel send per delivered row to a later measurement. Nothing is measured here
    /// and nothing follows: the probe disconnects immediately after its last check, which drops the
    /// callbacks with the connection.
    pub(crate) fn observe_control_activity_latest_by_control_view(
        &self,
        events: mpsc::Sender<ViewEventShape>,
    ) {
        let table = self.conn.db.control_activity_latest_by_control_view();
        table.on_insert({
            let events = events.clone();
            move |_ctx, _row| deliver(&events, ViewEventShape::Insert)
        });
        table.on_update({
            let events = events.clone();
            move |_ctx, _old, _new| deliver(&events, ViewEventShape::Update)
        });
        table.on_delete(move |_ctx, _row| deliver(&events, ViewEventShape::Delete));
    }

    /// Record a Control's **first** activity through the atomic registry reducer — the
    /// `ControlRegistry` candidate's identity-establishing write.
    ///
    /// Copies [`Self::insert_control_activity`] exactly, over `record_first_control_activity`, so
    /// every candidate's seeding waits on the same confirmation primitive.
    ///
    /// The module refuses a duplicate registry key or activity id by **returning** an error rather
    /// than panicking, so the refusal arrives at [`ReducerCompletion::ReducerFailed`] carrying the
    /// module's own text — the offending key named. A panicking reducer traps the instance and would
    /// arrive as [`ReducerCompletion::Internal`] with "The instance encountered a fatal error"
    /// instead, losing the message the fail-loud proofs assert against. Both abort the transaction;
    /// only one of them says why.
    pub(crate) fn record_first_control_activity(
        &self,
        id: u64,
        control_uuid: u64,
        user_identity: Identity,
        ts: Timestamp,
    ) -> Result<()> {
        self.await_reducer(|cb| {
            self.conn.reducers.record_first_control_activity_then(
                id,
                control_uuid,
                user_identity,
                ts,
                cb,
            )
        })
        .with_context(|| format!("recording first control activity id={id}"))
    }

    /// Record a **subsequent** activity through the atomic registry reducer.
    ///
    /// **Takes no `user_identity`**, unlike its first-activity counterpart, because the module
    /// derives it from the registry row. A caller cannot re-attribute a control's activity through
    /// this method, which is what makes the `control_uuid -> user_identity` dependency unreachable
    /// to violate rather than merely unviolated — the same reason
    /// [`Self::update_entity_owner`] takes no `owner`.
    pub(crate) fn record_control_activity(
        &self,
        id: u64,
        control_uuid: u64,
        ts: Timestamp,
    ) -> Result<()> {
        self.await_reducer(|cb| {
            self.conn
                .reducers
                .record_control_activity_then(id, control_uuid, ts, cb)
        })
        .with_context(|| format!("recording control activity id={id}"))
    }

    /// Subscribe to `control_registry_all_view` and block until applied — Arm A's subscription.
    pub(crate) fn subscribe_control_registry_all_view(&self) -> Result<()> {
        self.subscribe_and_await_applied(format!("SELECT * FROM {TABLE_CONTROL_REGISTRY_ALL_VIEW}"))
    }

    /// Read Arm A's currently-subscribed rows out of the live client cache, issuing no new
    /// subscription — the reproducer re-reads this after every phase.
    pub(crate) fn read_control_registry_all_view(&self) -> Vec<ControlRegistry> {
        self.conn.db.control_registry_all_view().iter().collect()
    }

    /// Subscribe to the `control_registry` base table and block until applied — Arm A's matched
    /// direct public-table Control.
    pub(crate) fn subscribe_control_registry(&self) -> Result<()> {
        self.subscribe_and_await_applied(format!("SELECT * FROM {TABLE_CONTROL_REGISTRY}"))
    }

    /// Read the `control_registry` base table out of the live client cache, issuing no new
    /// subscription.
    pub(crate) fn read_control_registry(&self) -> Vec<ControlRegistry> {
        self.conn.db.control_registry().iter().collect()
    }

    /// Subscribe to the `control_activity` base table and block until applied — the audit-history
    /// composition control for Arm B.
    pub(crate) fn subscribe_control_activity(&self) -> Result<()> {
        self.subscribe_and_await_applied(format!("SELECT * FROM {TABLE_CONTROL_ACTIVITY}"))
    }

    /// Read the `control_activity` base table out of the live client cache, issuing no new
    /// subscription.
    pub(crate) fn read_control_activity(&self) -> Vec<ControlActivity> {
        self.conn.db.control_activity().iter().collect()
    }

    /// Seed one `indexed_control_activity` row via its confirmed insertion reducer — the calibration
    /// pilot's seeding primitive for **both** of its populations.
    ///
    /// Copies [`Self::insert_control_activity`] exactly, over `insert_indexed_control_activity`, so
    /// the indexed arm's seeding waits on the same confirmation primitive as every other candidate's
    /// and cannot diverge in how it waits. The module's two inserters take an identical argument
    /// list for exactly this reason.
    ///
    /// **`user_identity` is a parameter and is not derived from the sender**, which is what lets one
    /// connection seed rows owned by an identity it is not connected as — the unrelated population
    /// the sender-scoped arm must *not* return. `ts` is likewise explicit rather than clock-read, so
    /// a seeded row is reproducible from the frozen constants alone.
    pub(crate) fn insert_indexed_control_activity(
        &self,
        id: u64,
        ts: Timestamp,
        control_uuid: u64,
        user_identity: Identity,
    ) -> Result<()> {
        self.await_reducer(|cb| {
            self.conn
                .reducers
                .insert_indexed_control_activity_then(id, ts, control_uuid, user_identity, cb)
        })
        .with_context(|| format!("seeding indexed_control_activity id={id}"))
    }

    /// Read the indexed sender-scoped view's currently-subscribed rows out of the live client cache,
    /// issuing no new subscription — the calibration pilot's measured arm.
    pub(crate) fn read_indexed_control_activity_sender_view(&self) -> Vec<IndexedControlActivity> {
        self.conn
            .db
            .indexed_control_activity_sender_view()
            .iter()
            .collect()
    }

    /// Read the `indexed_control_activity` base table out of the live client cache, issuing no new
    /// subscription — the calibration pilot's composition witness.
    pub(crate) fn read_indexed_control_activity(&self) -> Vec<IndexedControlActivity> {
        self.conn.db.indexed_control_activity().iter().collect()
    }

    /// The calibration pilot's **paced append** channel: one outstanding single-row append at a
    /// time, each sample stopped when its new row becomes visible in the subscriber's sender-scoped
    /// cache.
    ///
    /// The append-only analogue of [`Self::measure_paced_visible_batch`], and separate from it
    /// deliberately. That channel measures an *update* to a row that already exists, cycling a fixed
    /// key set and stopping on `on_update`; this one measures the production-shaped write the spec
    /// freezes for site 4 — a brand-new row appearing — and so must stop on `on_insert` and match a
    /// key that did not exist when the sample began. Widening the existing channel to serve both
    /// would put a mode switch inside a frozen measured path whose recorded evidence must not move.
    ///
    /// **One observer serves the whole batch, and its removal is enqueued before this returns on
    /// every path**, so nothing that runs afterwards on this connection keeps paying a channel send
    /// per delivered row.
    ///
    /// **No append can become visible before its observer is installed, and the proof is
    /// same-channel FIFO plus outbound causality — not any claim about when mutations are drained.**
    /// This connection runs `run_threaded`, whose loop is `advance_one_message_blocking` ->
    /// `get_message` (`db_connection.rs:594-628,634-640,665-674`): it prefers a ready local mutation,
    /// but otherwise `tokio::select!`s, so it emphatically does *not* drain pending mutations before
    /// every websocket message. The ordering that does hold:
    ///
    /// - `on_insert` installs nothing; it queues `PendingMutation::AddInsertCallback` through
    ///   `queue_mutation` (`client_cache.rs:451-453,461`);
    /// - the generated `insert_indexed_control_activity_then` reaches
    ///   `invoke_reducer_with_callback`, which queues `PendingMutation::InvokeReducerWithCallback`
    ///   (`db_connection.rs:747-762`);
    /// - both land on the *same* unbounded MPSC channel — the table handle's sender is a clone of
    ///   the connection's `pending_mutations_send` (`db_connection.rs:738-743`) — which is FIFO;
    /// - the connection thread therefore applies the callback registration strictly before it
    ///   applies the reducer invocation, and only the latter emits the outbound `CallReducer`.
    ///
    /// So the server has not even been asked to perform the append until the observer is installed,
    /// and no response or row update can precede its own request. The registration is queued before
    /// the first `sample_paced_appends` iteration, so this holds for every sample including the
    /// first.
    ///
    /// **The seeded snapshot is excluded for a separate reason.** The arm's subscription-applied
    /// callback — which is what unblocks `subscribe_untimed` — and that subscription's row callbacks
    /// are delivered while the connection thread processes one websocket message, and a locally
    /// queued mutation is a *different* message handled in a later loop iteration. A registration
    /// queued from the harness thread therefore cannot be spliced into the middle of the snapshot's
    /// callbacks, and the seeded own slice cannot reach this observer.
    ///
    /// **Removal is queued, not immediate** (`client_cache.rs:473-477`), so a row callback for an
    /// already-selected update may still run after this method returns and attempt one send. That is
    /// harmless and is the accepted observer's behaviour too: the channel is local to this call and
    /// dropped on return, and `deliver` models a dropped receiver explicitly.
    ///
    /// **A stopped batch returns its completed samples**, not a bare error, because a paced append
    /// batch that fails at sample 400 holds four hundred genuine intervals. See
    /// [`PacedAppendFailure`].
    pub(crate) fn measure_paced_appends(
        &self,
        appends: &[PacedAppend],
    ) -> std::result::Result<Vec<LatencySample>, PacedAppendFailure> {
        let (tx, rx) = mpsc::channel::<PacedAppendMessage>();
        let observer = self.conn.db.indexed_control_activity_sender_view().on_insert({
            let visible_tx = tx.clone();
            // The clock is read as this closure's first statement, on the SDK's callback thread.
            // What precedes it inside the callback is the generated handle's own row reference;
            // everything after it — the primary-key read, the channel send, this thread's wake — is
            // outside the sample.
            move |_ctx, row| {
                let observed_at = Instant::now();
                deliver(
                    &visible_tx,
                    PacedAppendMessage::Visible {
                        id: row.id,
                        observed_at,
                    },
                );
            }
        });

        let sampled = self.sample_paced_appends(&tx, &rx, appends);

        self.conn
            .db
            .indexed_control_activity_sender_view()
            .remove_on_insert(observer);
        sampled
    }

    /// Walk the paced append batch with the observer already registered.
    ///
    /// Split from [`Self::measure_paced_appends`] so the observer's removal covers every path out of
    /// this walk, early returns included, without a `Drop` guard — the split
    /// [`Self::sample_paced_batch`] exists for, for the same reason.
    ///
    /// Each sample's interval is issue-to-visible: the clock starts before the generated issue call
    /// and stops **in the observer callback** that reported *this* append's key, never when this
    /// thread wakes. Every reducer argument is prebuilt — the whole batch's rows were built before
    /// the first sample — so only the timestamp capture and the generated issue call fall inside the
    /// interval, plus the SDK's own unavoidable `Box::new(callback)` made within it.
    fn sample_paced_appends(
        &self,
        tx: &mpsc::Sender<PacedAppendMessage>,
        rx: &mpsc::Receiver<PacedAppendMessage>,
        appends: &[PacedAppend],
    ) -> std::result::Result<Vec<LatencySample>, PacedAppendFailure> {
        let mut samples = Vec::with_capacity(appends.len());

        for (index, append) in appends.iter().enumerate() {
            // Preregistered experimental pacing: a fixed delay *between* completed samples, outside
            // every measured interval. Applied before every sample except the first (nothing
            // precedes it) and never after the last — the exact protocol the paced update channel
            // applies, and the one the recorded `paced_sample_delay_ms` method fact names.
            if index != 0 {
                sleep(Duration::from_millis(PACED_SAMPLE_DELAY_MS));
            }

            let PacedAppend {
                id,
                ts,
                control_uuid,
                user_identity,
            } = *append;
            let paced_tx = tx.clone();
            let start = Instant::now();
            if let Err(e) = self.conn.reducers.insert_indexed_control_activity_then(
                id,
                ts,
                control_uuid,
                user_identity,
                paced_append_callback(index, id, paced_tx),
            ) {
                return Err(PacedAppendFailure {
                    samples,
                    failure: MeasuredStepFailure::Infrastructure(anyhow!(
                        "issuing paced indexed_control_activity append index {index}: {e:?}"
                    )),
                });
            }

            // Checked rather than `+`, but an invariant rather than an outcome. The append has
            // already been issued and may already have mutated the database, so there is no
            // truthful "this batch stopped" record to return from here; and a monotonic clock too
            // near its representable end is a property of the harness's host, not something the
            // experiment observed. The interval's start stays immediately before the issue, which is
            // what the estimand requires, and the deadline is derived from it after.
            let deadline = start
                .checked_add(PACED_SAMPLE_TIMEOUT)
                .expect("monotonic sample deadline must be representable");

            match await_visible_append(rx, id, start, deadline) {
                Ok(sample) => samples.push(sample),
                Err(failure) => return Err(PacedAppendFailure { samples, failure }),
            }
        }

        Ok(samples)
    }

    /// Register all three row callbacks on Arm A, reporting each delivery's [`ViewEventShape`].
    ///
    /// Same reasoning as
    /// [`Self::observe_control_activity_latest_by_control_view`]: the repeat-activity round replaces
    /// each control's row under an unchanged `control_uuid` key, and whether the server reports that
    /// as an in-place update or a delete plus an insert is an observation the spec requires be
    /// recorded rather than presumed. The callbacks are likewise never removed — nothing is measured
    /// and the reproducer disconnects immediately afterward.
    pub(crate) fn observe_control_registry_all_view(&self, events: mpsc::Sender<ViewEventShape>) {
        let table = self.conn.db.control_registry_all_view();
        table.on_insert({
            let events = events.clone();
            move |_ctx, _row| deliver(&events, ViewEventShape::Insert)
        });
        table.on_update({
            let events = events.clone();
            move |_ctx, _old, _new| deliver(&events, ViewEventShape::Update)
        });
        table.on_delete(move |_ctx, _row| deliver(&events, ViewEventShape::Delete));
    }

    /// Subscribe to the `entity_owner` base table and block until its initial snapshot is applied —
    /// the matched **direct public-table control** for the `EntityOwnerSenderView` candidate. The
    /// same subscription shape as the Arm's, differing only in naming the base table instead of the
    /// sender-scoped view, so the two roles' plumbing cannot diverge.
    pub(crate) fn subscribe_entity_owner(&self) -> Result<()> {
        self.subscribe_and_await_applied(format!("SELECT * FROM {TABLE_ENTITY_OWNER}"))
    }

    /// Read the `entity_owner` base table's currently-subscribed rows out of the live client cache,
    /// issuing no new subscription.
    pub(crate) fn read_entity_owner(&self) -> Vec<EntityOwner> {
        self.conn.db.entity_owner().iter().collect()
    }

    /// Register the campaign's three delivery callbacks on this role's measured target — the
    /// [`Self::register_dose_events`] seam for this campaign, supplying the private connection so the
    /// counted table cannot drift from the subscribed one.
    pub(crate) fn register_delivery_callbacks(
        &self,
        target: MeasuredTarget,
        counters: DeliveryCounters,
    ) {
        target.register_delivery_callbacks(&self.conn, counters);
    }

    /// Read this role's measured target out of the live client cache, issuing no new subscription.
    pub(crate) fn read_measured_target(&self, target: MeasuredTarget) -> Vec<EntityOwner> {
        target.read_back(&self.conn)
    }

    /// The campaign's **cold subscription** channel (E3): subscribe, block until applied, retain the
    /// handle, and time the interval from its own issue.
    ///
    /// Measured once per attempt — a second subscription on this connection is no longer cold, which
    /// is why the frozen order runs it first.
    pub(crate) fn subscribe_measured_target(
        &self,
        target: MeasuredTarget,
    ) -> std::result::Result<(SubscriptionHandle, LatencySample), MeasuredStepFailure> {
        let origin = Instant::now();
        self.subscribe_retained_from(target, origin)
    }

    /// The **cold subscription** channel (E3) for a site that supplies its own query: subscribe to
    /// `sql`, block until applied, retain the handle, and time from an origin anchored here.
    ///
    /// **Deliberately not shared with [`Self::subscribe_retained_from`]**, which serves the dormant
    /// campaign. The two want the query built at different points — the campaign builds its own
    /// inside the interval, as the evidence it already recorded was collected, while a site builds
    /// its query and its diagnostic wording *before* anchoring — and one helper cannot hold both
    /// without re-timing whichever caller it does not suit. A site arriving later must not perturb a
    /// frozen channel's recorded shape, so the duplication is the point: taking `String` rather than
    /// a [`MeasuredTarget`] is what makes it impossible for this function to build a query inside its
    /// own interval, and impossible for the campaign's to build one outside its own.
    ///
    /// **What is inside the interval.** The origin is anchored here, with `sql` and `description`
    /// already built — so the anchor precedes every step below, and none of them is "the issue".
    ///
    /// Harness-side first: one `mpsc` channel, one clone of its sender, and the two closures, which
    /// are built here but boxed by the SDK.
    ///
    /// Then the SDK's own work, all still inside the interval: `subscription_builder` constructs a
    /// `SubscriptionBuilder`, cloning the connection context; `on_applied` and `on_error` each box
    /// their callback; and `subscribe` allocates a query-set id, converts the query into `Box<str>`
    /// and collects it into a boxed slice — work proportional to the query, so it differs per target
    /// rather than being a fixed residue — clones the pending-mutation sender, builds the
    /// subscription state and handle, clones that handle, and **enqueues** a `PendingMutation` on an
    /// unbounded channel.
    ///
    /// What `subscribe` returns to us is therefore an enqueue, not a transmission: the SDK's
    /// background path performs the network work, and once the mutation is queued that path may
    /// already be running. The source proves the ordering of the enqueue, not a wall-clock claim
    /// about what the socket has done by the time this call returns. The server's materialization
    /// and the cache apply follow, and the interval ends inside `on_applied` as its first statement.
    ///
    /// Measured once per attempt: a second subscription on this connection is no longer cold.
    pub(crate) fn subscribe_cold_target(
        &self,
        sql: String,
        description: String,
    ) -> std::result::Result<(SubscriptionHandle, LatencySample), MeasuredStepFailure> {
        let origin = Instant::now();
        let (applied_tx, applied_rx) =
            mpsc::channel::<std::result::Result<LatencySample, String>>();
        let subscription = self
            .conn
            .subscription_builder()
            .on_applied({
                let applied_tx = applied_tx.clone();
                move |_ctx| {
                    let applied = LatencySample::from_elapsed(origin.elapsed());
                    deliver(&applied_tx, Ok(applied));
                }
            })
            .on_error(move |_ctx, err| {
                deliver(&applied_tx, Err(format!("{err:?}")));
            })
            .subscribe([sql]);

        let applied = match applied_rx.recv_timeout(SUBSCRIPTION_TIMEOUT) {
            Ok(applied) => applied,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(MeasuredStepFailure::Timeout(anyhow!(
                    "the subscription to {description} was not applied within \
                     {SUBSCRIPTION_TIMEOUT:?}"
                )))
            }
            // Not an elapsed bound: both senders live in callbacks the subscription owns, so a
            // disconnect means the SDK dropped them without applying or erroring.
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(MeasuredStepFailure::Infrastructure(anyhow!(
                    "the applied channel for the subscription to {description} disconnected \
                     without reporting either an applied snapshot or an error"
                )))
            }
        };
        let applied = applied.map_err(|msg| {
            MeasuredStepFailure::Infrastructure(anyhow!("subscription failed: {msg}"))
        })?;

        Ok((subscription, applied))
    }

    /// Subscribe to `sql`, block until its initial snapshot is applied, and **return the retained
    /// handle**. Untimed.
    ///
    /// The unmeasured counterpart to [`Self::subscribe_cold_target`], for the validation
    /// subscriptions a site issues *after* its timed interval has closed. Taking the query rather
    /// than naming a relation lets the caller draw both from one place, so the relation whose
    /// cardinality is validated is necessarily the relation that was subscribed.
    ///
    /// **The handle is returned rather than dropped, unlike
    /// [`Self::subscribe_and_await_applied`].** A site validating a multi-relation composition holds
    /// several subscriptions live at once and reads all their caches together, so the handles must
    /// outlive the reads. In pinned SDK 2.7.0 that is belt and braces rather than a fix:
    /// `SubscriptionHandleImpl` has no `Drop` impl, and unsubscription is only through
    /// `unsubscribe`/`unsubscribe_then`, which consume `self` — so a dropped handle leaves its
    /// subscription live. Relying on the absence of a `Drop` impl in an undocumented internal type
    /// is not a property worth resting a composition check on, and a caller holding the handle does
    /// not have to.
    ///
    /// Reaching the applied callback is part of the result: a query the server refused fails here
    /// rather than returning an empty cache and being read as a composition mismatch.
    pub(crate) fn subscribe_untimed(&self, sql: String) -> Result<SubscriptionHandle> {
        self.subscribe_retained_untimed(sql)
    }

    /// The campaign's **reconnect** channel (E4): release this connection, establish a new one
    /// presenting the *same* token, re-register `target`'s delivery callbacks, and resubscribe.
    ///
    /// **The measured interval excludes the teardown and includes everything after it** — the
    /// transport handshake, the identity and token round trip, the three callback registrations, and
    /// the snapshot. A proactive teardown is the harness's doing, not part of the production
    /// reconnect being estimated; excluding it is what makes E4 a distinct estimand from E3, which
    /// times a subscription on a connection that already exists. The counter handles are cloned
    /// before the clock, leaving only the SDK's own per-callback boxing inside.
    ///
    /// **Consumes `self`, and releases before it reconnects.** Moving `conn` and `handle` out of a
    /// `&mut self` would need an `Option` or a dummy to leave behind, and connecting first would put
    /// two live connections on one database — which the SDK's process-global registry labels only by
    /// database name, so both would sum into the series the measured window reads. Every failure's
    /// surviving state is named by [`ReconnectFailure`].
    ///
    /// The old subscription handle is not carried across: pinned SDK 2.7.0 does not mark an applied
    /// subscription inactive when its connection ends, so retaining it would report two active
    /// subscriptions where one exists.
    pub(crate) fn reconnect_preserving_token(
        self,
        target: MeasuredTarget,
        rows: &Arc<crate::view_read_set_campaign::subscriber_delivery::subscriber_row_counter::SubscriberRowCounter>,
    ) -> std::result::Result<(Self, SubscriptionHandle, LatencySample), ReconnectFailure> {
        let server_url = self.server_url.clone();
        let database_name = self.database_name.clone();
        let token = self.token.clone();
        let counters = DeliveryCounters::of(rows);

        self.disconnect().map_err(|error| {
            ReconnectFailure::ReleaseFailed(
                error.context("releasing the measured subscriber before its reconnect"),
            )
        })?;

        let origin = Instant::now();
        let client = match Self::establish(&server_url, &database_name, Some(token)) {
            Ok(client) => client,
            // Both clientless post-conditions land on the same variant, because the caller's
            // obligation is identical: nothing was left running.
            Err(ConnectFailure::NotBuilt(cause) | ConnectFailure::ReleasedAfterStart(cause)) => {
                return Err(ReconnectFailure::NotConnected(cause))
            }
            Err(ConnectFailure::ReleasedWithError(error)) => {
                return Err(ReconnectFailure::ReleaseFailed(error.context(
                    "releasing the half-connected replacement subscriber after its handshake \
                     failed",
                )))
            }
        };
        target.register_delivery_callbacks(&client.conn, counters);
        match client.subscribe_retained_from(target, origin) {
            Ok((subscription, applied)) => Ok((client, subscription, applied)),
            Err(failure) => Err(ReconnectFailure::NotResubscribed { client, failure }),
        }
    }

    /// The campaign's **paced visible-apply** channel (E2): one outstanding write at a time, each
    /// sample stopped when its change is observable in the subscriber cache.
    ///
    /// One observer serves the whole batch and is **removed before returning on every path**, so the
    /// saturated channel never pays a channel send per delivered row. The delivery counters stay
    /// registered — they count the whole measured window.
    pub(crate) fn measure_paced_visible_batch(
        &self,
        target: MeasuredTarget,
        schedule: MutationSchedule,
    ) -> std::result::Result<RawLatencies, MeasuredStepFailure> {
        let (tx, rx) = mpsc::channel::<PacedMessage>();
        let observer = target.observe_visible_updates(&self.conn, {
            let visible_tx = tx.clone();
            // The clock is read as this closure's first statement, on the SDK's callback thread.
            // What precedes it inside the callback is the generated handle's own `u64` primary-key
            // read; everything after it — the channel send, this thread's wake — is outside the
            // sample.
            move |entity_uuid| {
                let observed_at = Instant::now();
                deliver(
                    &visible_tx,
                    PacedMessage::Visible {
                        entity_uuid,
                        observed_at,
                    },
                );
            }
        });

        let sampled = self.sample_paced_batch(&tx, &rx, schedule);

        observer.remove(&self.conn);
        sampled
    }

    /// The campaign's **saturated queue-growth** channel (E1): issue the whole batch back-to-back
    /// without awaiting any of it, then barrier and seal an issue-ordered [`SaturatedTimingBatch`].
    ///
    /// Runs last in the frozen order, where its queue pressure cannot perturb an unmeasured channel.
    ///
    /// **One origin per batch**, so issue spacing and confirmation ordering are comparable across
    /// writes and the FIFO reading stays falsifiable. Each offset is captured on the side it
    /// describes. The key, payload, and `Sender` clone are built before the issue offset is captured,
    /// as the historical measured batch builds them; whatever overhead remains is visible in the
    /// recorded offsets rather than assumed away.
    pub(crate) fn measure_saturated_batch(
        &self,
        schedule: MutationSchedule,
    ) -> std::result::Result<SaturatedTimingBatch, MeasuredStepFailure> {
        let (tx, rx) = mpsc::channel::<SaturatedMessage>();
        let origin = Instant::now();

        for (index, write) in schedule.writes().into_iter().enumerate() {
            let entity_uuid = schedule.target_key(write);
            let record = schedule.payload(write);
            let saturated_tx = tx.clone();
            let issue_offset_nanos = origin.elapsed().as_nanos();
            self.conn
                .reducers
                .update_entity_owner_then(
                    entity_uuid,
                    record,
                    saturated_callback(index, origin, issue_offset_nanos, saturated_tx),
                )
                .map_err(|e| {
                    MeasuredStepFailure::Infrastructure(anyhow!(
                        "issuing saturated entity_owner update index {index}: {e:?}"
                    ))
                })?;
        }

        // Anchor the one whole-batch deadline now that issuing is done, matching the historical
        // measured batch. `tx` outlives the barrier, so the channel cannot disconnect mid-batch.
        let deadline = Instant::now() + MEASURED_BATCH_TIMEOUT;
        collect_saturated_batch(rx, deadline)
    }

    /// Subscribe to `target`, block until applied, retain the handle, and measure from `origin`.
    ///
    /// **The origin is the caller's**, because the two apply channels measure different intervals
    /// ending at the same event: E3's begins at this subscription's own issue, E4's before its
    /// connection existed. Anchoring here would collapse them into one estimand.
    ///
    /// Stopping at `on_applied` is exactly "the initial snapshot is in cache": the pinned SDK writes
    /// the cache, then runs the applied callback, then the row callbacks.
    ///
    /// **The interval ends inside the applied callback**, not when this thread wakes from the
    /// barrier. `origin.elapsed()` is read as that callback's first statement, on the SDK's own
    /// thread, so the channel hand-off and this thread's scheduling are outside the sample — the
    /// discipline the historical measured batch already follows by reading its elapsed inside the
    /// confirmation callback. Waking first would add one channel wake to every apply sample, and an
    /// additive residue biases the endpoint factor `T = S_last / S_first` toward one.
    ///
    /// For E3 the caller anchors immediately before this call, so one `mpsc` channel, two boxed
    /// callbacks, and the query fall inside the interval. The channel and callback setup is common
    /// to both channels, but the query is not a fixed residue: `target.subscription_sql()` is
    /// evaluated here, inside the interval, and both it and the SDK's conversion of its result
    /// scale with the query, so they differ by target. All of it runs against a snapshot of
    /// thousands of rows over a socket, and is disclosed rather than contorted out of an API the two
    /// share. The measured endpoint and the evaluation order are untouched: they remain exactly the
    /// accepted implementation's.
    fn subscribe_retained_from(
        &self,
        target: MeasuredTarget,
        origin: Instant,
    ) -> std::result::Result<(SubscriptionHandle, LatencySample), MeasuredStepFailure> {
        let (applied_tx, applied_rx) =
            mpsc::channel::<std::result::Result<LatencySample, String>>();
        let subscription = self
            .conn
            .subscription_builder()
            .on_applied({
                let applied_tx = applied_tx.clone();
                move |_ctx| {
                    let applied = LatencySample::from_elapsed(origin.elapsed());
                    deliver(&applied_tx, Ok(applied));
                }
            })
            .on_error(move |_ctx, err| {
                deliver(&applied_tx, Err(format!("{err:?}")));
            })
            .subscribe([target.subscription_sql()]);

        let applied = match applied_rx.recv_timeout(SUBSCRIPTION_TIMEOUT) {
            Ok(applied) => applied,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(MeasuredStepFailure::Timeout(anyhow!(
                    "the subscription to {target:?} was not applied within {SUBSCRIPTION_TIMEOUT:?}"
                )))
            }
            // Not an elapsed bound: both senders live in callbacks the subscription owns, so a
            // disconnect means the SDK dropped them without applying or erroring.
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(MeasuredStepFailure::Infrastructure(anyhow!(
                    "the applied channel for the subscription to {target:?} disconnected without \
                     reporting either an applied snapshot or an error"
                )))
            }
        };
        let applied = applied.map_err(|msg| {
            MeasuredStepFailure::Infrastructure(anyhow!("subscription failed: {msg}"))
        })?;

        Ok((subscription, applied))
    }

    /// Walk the paced batch with the observer already registered.
    ///
    /// Split from [`Self::measure_paced_visible_batch`] so the observer's removal covers every path
    /// out of this walk, early returns included, without a `Drop` guard.
    ///
    /// Each sample's interval is issue-to-visible: the clock starts before the generated issue call
    /// and stops **in the observer callback** that reported *this* write's key, never when this
    /// thread wakes. Key, payload, and `Sender` clone are built before the clock.
    fn sample_paced_batch(
        &self,
        tx: &mpsc::Sender<PacedMessage>,
        rx: &mpsc::Receiver<PacedMessage>,
        schedule: MutationSchedule,
    ) -> std::result::Result<RawLatencies, MeasuredStepFailure> {
        let mut samples = Vec::with_capacity(CHANNEL_SAMPLE_COUNT_USIZE);

        for (index, write) in schedule.writes().into_iter().enumerate() {
            // Preregistered experimental pacing: a fixed delay *between* completed samples, outside
            // every measured interval. Applied before every sample except the first (nothing
            // precedes it) and never after the last — the exact protocol the historical dose ladder
            // applies between its batches.
            if index != 0 {
                sleep(Duration::from_millis(PACED_SAMPLE_DELAY_MS));
            }

            let entity_uuid = schedule.target_key(write);
            let record = schedule.payload(write);
            let paced_tx = tx.clone();
            let start = Instant::now();
            self.conn
                .reducers
                .update_entity_owner_then(entity_uuid, record, paced_callback(index, paced_tx))
                .map_err(|e| {
                    MeasuredStepFailure::Infrastructure(anyhow!(
                        "issuing paced entity_owner update index {index}: {e:?}"
                    ))
                })?;

            let sample =
                await_visible_update(rx, entity_uuid, start, start + PACED_SAMPLE_TIMEOUT)?;
            samples.push(sample);
        }

        RawLatencies::sealed(samples).map_err(MeasuredStepFailure::Infrastructure)
    }

    /// Measure one [`BATCH_SIZE`]-write `entity_owner` batch, returning the issue-ordered
    /// [`RawLatencies`] — the `EntityOwnerSenderView` candidate's measured-write primitive.
    ///
    /// Structurally identical to [`Self::measure_writes`]' phase B, over `insert_entity_owner`
    /// instead of the historical reducers and with no prerequisite phase (an `entity_owner` row has
    /// no precondition row). The writes take consecutive `entity_uuid`s from `key_base` so the batch
    /// occupies a contiguous, caller-allocated key range that cannot overlap another rung's.
    ///
    /// The measured interval is kept free of harness-side allocation exactly as the historical batch
    /// does: the key, the owned `record` payload, and the callback's `Sender` clone are all prebuilt
    /// *before* `Instant::now()`, so only the timestamp capture and the generated issue call fall
    /// inside it. The barrier, the callback, and the whole-batch deadline are the historical
    /// [`collect_measured_batch`] / [`measured_callback`] / [`MEASURED_BATCH_TIMEOUT`], reused
    /// unchanged.
    pub(crate) fn measure_entity_owner_batch(
        &self,
        key_base: u64,
        owner: Identity,
    ) -> Result<RawLatencies> {
        let (tx, rx) = mpsc::channel::<MeasuredMessage>();

        for index in 0..BATCH_SIZE_USIZE {
            let offset = u64::try_from(index).expect("a batch index fits u64");
            let entity_uuid = key_base
                .checked_add(offset)
                .expect("the Pilot's global key space must not overflow u64");
            let record = PILOT_ROW_PAYLOAD.to_string();
            let measured_tx = tx.clone();
            let start = Instant::now();
            self.conn
                .reducers
                .insert_entity_owner_then(
                    entity_uuid,
                    owner,
                    record,
                    measured_callback(index, start, measured_tx),
                )
                .map_err(|e| anyhow!("issuing measured entity_owner write index {index}: {e:?}"))?;
        }

        // Anchor the one whole-batch deadline now that issuing is done, matching the historical
        // measured batch. `tx` outlives the barrier, so the channel cannot disconnect mid-batch and
        // every stall is a timeout.
        let deadline = Instant::now() + MEASURED_BATCH_TIMEOUT;
        collect_measured_batch(rx, deadline)
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
    /// - **Phase A ([`Self::confirm_prerequisites`])** issues all [`BATCH_SIZE`] `message_visibility`
    ///   prerequisite inserts back-to-back and barriers until *every* one has confirmed successfully.
    ///   A confirmed callback establishes that the prerequisite transaction completed successfully, so
    ///   once phase A returns every prerequisite transaction has finished and a later transaction can
    ///   observe its visibility row — no reliance on the relative execution order of two reducers on
    ///   the connection. This phase is entirely outside every measured interval.
    /// - **Phase B ([`Self::measure_writes`])** then issues all [`BATCH_SIZE`] measured writes
    ///   back-to-back — for Chronicle the `chronicle_message` inserts (the writes that always change
    ///   the direct control's subscribed `chronicle_message` table, and — under `OwnSliceGrowth` —
    ///   also supply the missing key the measured subscriber's own pair awaits), for the message
    ///   family the `message` inserts — barriers on their confirmed round trips, and seals. With phase
    ///   A already complete, no prerequisite issue is interleaved between two measured writes, so the
    ///   measured batch is uninterrupted.
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

        // Phase A: for a Chronicle dose, land and confirm every prerequisite message_visibility row
        // before any measured write is issued. The message family skips this phase entirely.
        if chronicle_pairs == BATCH_SIZE_USIZE {
            self.confirm_prerequisites(operations)?;
        }

        // Phase B: the Anton-shaped back-to-back measured batch (both families).
        self.measure_writes(operations)
    }

    /// Phase A of a Chronicle dose: issue every op's `message_visibility` prerequisite insert
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
            if let SeedOp::ChroniclePair { key, viewer } = *operation {
                let prerequisite_tx = tx.clone();
                self.conn
                    .reducers
                    .insert_message_visibility_then(
                        key,
                        viewer,
                        key,
                        prerequisite_callback(index, prerequisite_tx),
                    )
                    .map_err(|e| {
                        anyhow!(
                            "issuing prerequisite message_visibility write index {index}: {e:?}"
                        )
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
    /// the `chronicle_message` insert (whose prerequisite `message_visibility` row was already
    /// confirmed in phase A). No prerequisite issue is interleaved here, so the measured writes are genuinely
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
                SeedOp::ChroniclePair { key, .. } => {
                    let payload = ROW_PAYLOAD.to_string();
                    let measured_tx = tx.clone();
                    let start = Instant::now();
                    self.conn
                        .reducers
                        .insert_chronicle_message_then(
                            key,
                            payload,
                            measured_callback(index, start, measured_tx),
                        )
                        .map_err(|e| {
                            anyhow!("issuing measured chronicle_message write index {index}: {e:?}")
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
        self.subscribe_retained_untimed(sql).map(drop)
    }

    /// Subscribe to `sql`, block until applied, and return the handle.
    ///
    /// The body of [`Self::subscribe_and_await_applied`], which now discards the handle explicitly
    /// through this. Retaining it is what a caller validating several relations at once needs; the
    /// single-relation callers that came first genuinely have nothing to hold it for, and dropping
    /// it leaves the subscription live — pinned `SubscriptionHandleImpl` has no `Drop` impl, and both
    /// its unsubscribe methods consume `self`.
    fn subscribe_retained_untimed(&self, sql: String) -> Result<SubscriptionHandle> {
        let (applied_tx, applied_rx) = mpsc::channel::<std::result::Result<(), String>>();
        let subscription = self
            .conn
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
            .map_err(|msg| anyhow!("subscription failed: {msg}"))?;
        Ok(subscription)
    }

    /// Disconnect and join the message-processing thread, reporting every failure the release
    /// produced.
    ///
    /// Both steps always run and their errors aggregate, so an `Err` here means the connection ended
    /// abnormally rather than that it may still be running. See [`release_connection`].
    pub(crate) fn disconnect(self) -> Result<()> {
        let Self { conn, handle, .. } = self;
        let errors = release_connection(&conn, handle, ReleaseExpectation::Live);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(into_error(errors))
        }
    }

    /// Issue one reducer via `issue` and block until its confirmed completion callback fires,
    /// translating a reducer-returned error or an internal error into a failure.
    ///
    /// The unclassified wrapper over [`Self::await_reducer_classified`], for callers outside any
    /// measured window — the historical Pilot and Smoke seeding paths, which have no channel to
    /// classify for. The error it returns is the classified one's cause, so their text is the same.
    fn await_reducer(
        &self,
        issue: impl FnOnce(BoxedReducerCallback) -> spacetimedb_sdk::Result<()>,
    ) -> Result<()> {
        self.await_reducer_classified(issue)
            .map_err(MeasuredStepFailure::into_error)
    }

    /// Issue one reducer via `issue` and block until its completion callback fires, keeping the
    /// classified cause.
    ///
    /// Only the code that issued the reducer and owned the deadline can tell these apart, which is
    /// why the classification is made here rather than recovered from message text downstream:
    /// issuing failed on the transport, so [`MeasuredStepFailure::Infrastructure`]; the callback
    /// reported either failure, so [`MeasuredStepFailure::Application`]; the wait elapsed, so
    /// [`MeasuredStepFailure::Timeout`]; the sender was dropped without a callback, so
    /// `Infrastructure` again.
    fn await_reducer_classified(
        &self,
        issue: impl FnOnce(BoxedReducerCallback) -> spacetimedb_sdk::Result<()>,
    ) -> std::result::Result<(), MeasuredStepFailure> {
        let (done_tx, done_rx) = mpsc::channel::<ReducerCompletion>();
        let callback: BoxedReducerCallback = Box::new(move |_ctx, outcome| {
            let completion = match outcome {
                Ok(Ok(())) => ReducerCompletion::Confirmed,
                Ok(Err(msg)) => ReducerCompletion::ReducerFailed(msg),
                Err(internal) => ReducerCompletion::Internal(format!("{internal:?}")),
            };
            deliver(&done_tx, completion);
        });
        issue(callback)
            .map_err(|e| MeasuredStepFailure::Infrastructure(anyhow!("issuing reducer: {e:?}")))?;
        await_reducer_completion(done_rx)
    }
}

/// The awaited-reducer barrier: block for one completion and classify what arrived.
///
/// Factored out of [`ConnectedClient::await_reducer_classified`] so it depends on nothing but the
/// channel — no live server, which is what makes both application classifications provable.
///
/// Both callback failures are the application's answer, matching every other channel in this file:
/// [`collect_saturated_batch`] and [`await_visible_update`] classify a `Failed` message the same way
/// whichever of the two produced it. Only the wait itself is an elapsed bound, and only a sender
/// dropped without ever running the callback is infrastructure — the two `recv_timeout` errors that
/// a single classification would conflate.
fn await_reducer_completion(
    rx: mpsc::Receiver<ReducerCompletion>,
) -> std::result::Result<(), MeasuredStepFailure> {
    let completion = match rx.recv_timeout(REDUCER_TIMEOUT) {
        Ok(completion) => completion,
        Err(waiting @ mpsc::RecvTimeoutError::Timeout) => {
            return Err(MeasuredStepFailure::Timeout(
                Error::new(waiting).context(AWAITING_REDUCER),
            ))
        }
        Err(waiting @ mpsc::RecvTimeoutError::Disconnected) => {
            return Err(MeasuredStepFailure::Infrastructure(
                Error::new(waiting).context(AWAITING_REDUCER),
            ))
        }
    };

    match completion {
        ReducerCompletion::Confirmed => Ok(()),
        ReducerCompletion::ReducerFailed(msg) => Err(MeasuredStepFailure::Application(anyhow!(
            "reducer returned an error: {msg}"
        ))),
        ReducerCompletion::Internal(msg) => Err(MeasuredStepFailure::Application(anyhow!(
            "internal error awaiting reducer: {msg}"
        ))),
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
                    "prerequisite message_visibility write index {index} failed: {error}"
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

/// Release one connection's transport and its message-processing thread, returning every failure the
/// release produced.
///
/// **The join runs even when the disconnect reports an error**, and that is what makes every
/// caller's "no client remains" claim true: a join returns only once the thread has ended, `Err`
/// meaning it ended by panicking. Dropping the handle on the error path would return while the
/// thread might still be running.
///
/// It terminates, from the pinned source: `DbConnectionImpl::disconnect` returns
/// [`spacetimedb_sdk::Error::Disconnected`] when the connection is no longer active and otherwise
/// queues its `Disconnect` mutation — those are its only outcomes. A failure therefore means the
/// socket already ended, which is exactly the condition `run_threaded`'s loop returns on; a success
/// means the loop has been asked to stop.
///
/// A panicked join is reported rather than swallowed: `run_threaded` panics only on an error its
/// loop does not classify as a normal disconnect.
fn release_connection(
    conn: &DbConnection,
    handle: JoinHandle<()>,
    expectation: ReleaseExpectation,
) -> Vec<Error> {
    let mut errors = Vec::new();

    match conn.disconnect() {
        Ok(()) => {}
        Err(spacetimedb_sdk::Error::Disconnected)
            if matches!(expectation, ReleaseExpectation::Unhandshaken) => {}
        Err(error) => errors.push(anyhow!("disconnecting client: {error:?}")),
    }

    if handle.join().is_err() {
        errors.push(anyhow!("client message-processing thread panicked"));
    }

    errors
}

/// E2's per-sample barrier: block until the subscriber cache reports an update to `target_key`, and
/// return that sample as the interval from `start` to **the instant its own observer callback ran**.
///
/// Factored out of the paced walk so it depends on nothing but the channel, the key, the sample's
/// `start`, and one absolute `deadline` — no live server.
///
/// **Updates to other keys are skipped, not rejected**, because the frozen schedule cycles the ten
/// owned keys: rejecting one would fail a healthy run, and stopping on one would time the wrong
/// write. A skipped message's instant is discarded with it: the sample is the *target* key's
/// endpoint, never the last instant seen.
///
/// The subtraction is checked. `start` is captured before the write is issued and `observed_at` in a
/// callback that can only run after it, so an endpoint preceding its own start is a broken monotonic
/// clock rather than a slow write, and is refused instead of saturating to a zero-length sample that
/// the channel's reduction would accept as a real observation.
///
/// That no earlier sample leaves an unread message naming this key is a pinned-SDK property, not a
/// property of this loop: the cache applies inserts and deletes by exact row bytes *before* pairing
/// survivors by primary key, so a whole-view refresh in which one row changed yields exactly one
/// update event. Paced samples are serial and each consumes its own.
///
/// A reducer failure arrives on the same channel, so a write whose visibility can never come is
/// reported as the application error it is rather than as a timeout.
fn await_visible_update(
    rx: &mpsc::Receiver<PacedMessage>,
    target_key: u64,
    start: Instant,
    deadline: Instant,
) -> std::result::Result<LatencySample, MeasuredStepFailure> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let message = match rx.recv_timeout(remaining) {
            Ok(message) => message,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(MeasuredStepFailure::Timeout(anyhow!(
                    "the paced update to entity_uuid {target_key} did not become visible in the \
                     subscriber cache within {PACED_SAMPLE_TIMEOUT:?}"
                )))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(MeasuredStepFailure::Infrastructure(anyhow!(
                    "the paced sample channel disconnected while awaiting entity_uuid \
                     {target_key}; the measuring thread holds a sender for the whole batch, so a \
                     disconnect means the harness dropped one"
                )))
            }
        };
        match message {
            PacedMessage::Visible {
                entity_uuid,
                observed_at,
            } if entity_uuid == target_key => {
                let elapsed = observed_at.checked_duration_since(start).ok_or_else(|| {
                    MeasuredStepFailure::Infrastructure(anyhow!(
                        "the observer saw entity_uuid {target_key} before its own write was \
                         issued, so the monotonic clock did not advance across the sample"
                    ))
                })?;
                return Ok(LatencySample::from_elapsed(elapsed));
            }
            PacedMessage::Visible { .. } => {}
            PacedMessage::Failed { index, error } => {
                return Err(MeasuredStepFailure::Application(anyhow!(
                    "paced write index {index} failed: {error}"
                )))
            }
        }
    }
}

/// The paced append channel's per-sample barrier: block until the subscriber cache reports the
/// insertion of `target_id`, and return that sample as the interval from `start` to **the instant its
/// own observer callback ran**.
///
/// Factored out of the append walk so it depends on nothing but the channel, the key, the sample's
/// `start`, and one absolute `deadline` — no live server.
///
/// **Every message is filtered by id, failures included, and that is the whole protocol.** Under the
/// pinned SDK this is defensive rather than load-bearing: one append yields a visibility or a
/// failure and never both, because a successful `ReducerResult` applies the update and then
/// completes the callback with a success this channel reports as silence, while any failure
/// completes the callback without applying an update (`db_connection.rs:195-221,223-254`). Filtering
/// failures by id is kept because it makes the rule total — a failure ends the sample it *names*,
/// so no unexpected protocol state can be read as the current sample's outcome — and because one
/// rule over one key is easier to check than two rules over two.
///
/// **Non-matching messages of either kind are skipped rather than rejected.** For visibilities this
/// copies [`await_visible_update`]'s protocol; the reasoning differs from that channel's and is worth
/// stating rather than inheriting. There, other keys arrive because the frozen schedule cycles ten of
/// them. Here every append is a fresh id, appends are serial, and the observer is registered after
/// the arm's initial snapshot has applied — so in a healthy batch the channel carries nothing but
/// this sample's insert. A message naming another id is therefore about a sample that has already
/// been sealed or has not yet been issued, and neither is this sample's outcome to report: an
/// unexpected row reaches the ledger through the composition verifier, and a genuinely absent
/// visibility ends this sample on its own deadline. Rejecting instead would let one stale message
/// terminate an attempt that was proceeding correctly. A skipped message's instant is discarded with
/// it: the sample is the *target* id's endpoint, never the last instant seen.
///
/// The subtraction is checked, as it is on the update channel: `start` is captured before the append
/// is issued and `observed_at` in a callback that can only run after it, so an endpoint preceding its
/// own start is a broken monotonic clock rather than a slow write, and is refused instead of
/// saturating to a zero-length sample.
///
/// A reducer failure arrives on the same channel, so an append whose visibility can never come is
/// reported as the application error it is rather than as a timeout.
fn await_visible_append(
    rx: &mpsc::Receiver<PacedAppendMessage>,
    target_id: u64,
    start: Instant,
    deadline: Instant,
) -> std::result::Result<LatencySample, MeasuredStepFailure> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let message = match rx.recv_timeout(remaining) {
            Ok(message) => message,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(MeasuredStepFailure::Timeout(anyhow!(
                    "the paced append of indexed_control_activity id {target_id} did not become \
                     visible in the subscriber cache within {PACED_SAMPLE_TIMEOUT:?}"
                )))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(MeasuredStepFailure::Infrastructure(anyhow!(
                    "the paced append channel disconnected while awaiting indexed_control_activity \
                     id {target_id}; the measuring thread holds a sender for the whole batch, so a \
                     disconnect means the harness dropped one"
                )))
            }
        };
        match message {
            PacedAppendMessage::Visible { id, observed_at } if id == target_id => {
                let elapsed = observed_at.checked_duration_since(start).ok_or_else(|| {
                    MeasuredStepFailure::Infrastructure(anyhow!(
                        "the observer saw indexed_control_activity id {target_id} before its own \
                         append was issued, so the monotonic clock did not advance across the sample"
                    ))
                })?;
                return Ok(LatencySample::from_elapsed(elapsed));
            }
            PacedAppendMessage::Failed { index, id, error } if id == target_id => {
                return Err(MeasuredStepFailure::Application(anyhow!(
                    "paced append index {index} of indexed_control_activity id {id} failed: {error}"
                )))
            }
            // Both skips, together, so the filtering rule reads as one rule over one key rather than
            // as a special case for failures.
            PacedAppendMessage::Visible { .. } | PacedAppendMessage::Failed { .. } => {}
        }
    }
}

/// E1's barrier: receive the saturated batch's callbacks into a slot-indexed
/// [`SaturatedTimingAccumulator`] and seal the timings in issue order once every write confirms.
///
/// Factored out of [`ConnectedClient::measure_saturated_batch`] so it depends on nothing but the
/// channel and one absolute `deadline` — no live server. A duplicate or out-of-range fire, an
/// inverted offset pair, or any [`SaturatedMessage::Failed`] fails loud; delivery order is never
/// assumed, and every receive draws from the one `deadline`, as the historical measured batch does.
fn collect_saturated_batch(
    rx: mpsc::Receiver<SaturatedMessage>,
    deadline: Instant,
) -> std::result::Result<SaturatedTimingBatch, MeasuredStepFailure> {
    let mut measured = SaturatedTimingAccumulator::new();

    while !measured.is_complete() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let message = match rx.recv_timeout(remaining) {
            Ok(message) => message,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(MeasuredStepFailure::Timeout(anyhow!(
                    "the saturated batch did not fully confirm within {MEASURED_BATCH_TIMEOUT:?} \
                     of its last issue"
                )))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(MeasuredStepFailure::Infrastructure(anyhow!(
                    "the saturated batch channel disconnected mid-batch; the issuing thread holds \
                     a sender for the whole batch, so a disconnect means the harness dropped one"
                )))
            }
        };
        match message {
            SaturatedMessage::Confirmed {
                index,
                issue_offset_nanos,
                confirmation_offset_nanos,
            } => {
                let timing =
                    SaturatedWriteTiming::observed(issue_offset_nanos, confirmation_offset_nanos)
                        .with_context(|| format!("recording saturated write index {index}"))
                        .map_err(MeasuredStepFailure::Infrastructure)?;
                measured
                    .record(index, timing)
                    .map_err(MeasuredStepFailure::Infrastructure)?;
            }
            SaturatedMessage::Failed { index, error } => {
                return Err(MeasuredStepFailure::Application(anyhow!(
                    "saturated write index {index} failed: {error}"
                )))
            }
        }
    }

    measured.seal().map_err(MeasuredStepFailure::Infrastructure)
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

/// Build the completion callback for one **paced** write at issue `index`.
///
/// Silent on success: the sample stops on visibility, which the observer signals, so a confirmation
/// message would be a second unread send per write on the path E2 measures. This exists for the case
/// visibility can never come, so the failure surfaces as an application error rather than expiring
/// against the deadline.
///
/// Unboxed, as [`measured_callback`] is.
fn paced_callback(
    index: usize,
    tx: mpsc::Sender<PacedMessage>,
) -> impl FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static {
    move |_ctx, outcome| match outcome {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => deliver(
            &tx,
            PacedMessage::Failed {
                index,
                error: format!("reducer returned an error: {msg}"),
            },
        ),
        Err(internal) => deliver(
            &tx,
            PacedMessage::Failed {
                index,
                error: format!("internal error awaiting reducer: {internal:?}"),
            },
        ),
    }
}

/// Build the completion callback for one **paced append** at issue `index`.
///
/// Silent on success, exactly as [`paced_callback`] is: the sample stops on the row's visibility,
/// which the observer signals, so a confirmation message would be a second unread send per append on
/// the path this channel measures. It exists for the case visibility can never come, so the failure
/// surfaces as the application error it is rather than expiring against the deadline.
///
/// Unboxed, as [`measured_callback`] is.
fn paced_append_callback(
    index: usize,
    id: u64,
    tx: mpsc::Sender<PacedAppendMessage>,
) -> impl FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static {
    move |_ctx, outcome| match outcome {
        Ok(Ok(())) => {}
        Ok(Err(msg)) => deliver(
            &tx,
            PacedAppendMessage::Failed {
                index,
                id,
                error: format!("reducer returned an error: {msg}"),
            },
        ),
        Err(internal) => deliver(
            &tx,
            PacedAppendMessage::Failed {
                index,
                id,
                error: format!("internal error awaiting reducer: {internal:?}"),
            },
        ),
    }
}

/// Build the completion callback for one **saturated** write at issue `index`, carrying its already
/// captured `issue_offset_nanos` so both halves of the pair travel together and no index lookup can
/// cross them. The issue offset cannot be recomputed here: this callback runs arbitrarily later.
///
/// Unboxed, as [`measured_callback`] is.
fn saturated_callback(
    index: usize,
    origin: Instant,
    issue_offset_nanos: u128,
    tx: mpsc::Sender<SaturatedMessage>,
) -> impl FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static {
    move |_ctx, outcome| {
        let message = match outcome {
            Ok(Ok(())) => SaturatedMessage::Confirmed {
                index,
                issue_offset_nanos,
                confirmation_offset_nanos: origin.elapsed().as_nanos(),
            },
            Ok(Err(msg)) => SaturatedMessage::Failed {
                index,
                error: format!("reducer returned an error: {msg}"),
            },
            Err(internal) => SaturatedMessage::Failed {
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
