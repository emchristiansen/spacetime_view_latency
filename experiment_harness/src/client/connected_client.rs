//! A connected measured-subscriber client: connect, seed, subscribe, read back.

use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use spacetimedb_sdk::__codegen::InternalError;
use spacetimedb_sdk::{DbContext, Identity};

use crate::dataset::seed_op::SeedOp;
use crate::dataset::subscribed_rows::SubscribedRows;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::module_artifact::bindings::{
    insert_chronicle_message, insert_message, insert_message_visibility, DbConnection,
    ReducerEventContext,
};
use crate::params::{CONFIRMED_READS, ROW_PAYLOAD};

/// Wait budget for the initial connection handshake (`on_connect` / `on_connect_error`).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Wait budget for one confirmed seeding reducer round trip. Generous relative to a single
/// fsync-confirmed insert; seeding is not the measured operation.
const REDUCER_TIMEOUT: Duration = Duration::from_secs(60);
/// Wait budget for the initial subscription snapshot to be applied.
const SUBSCRIPTION_TIMEOUT: Duration = Duration::from_secs(30);

/// The flattened outcome of an insertion reducer as delivered to its completion callback.
type ReducerOutcome = std::result::Result<std::result::Result<(), String>, InternalError>;
/// A boxed reducer completion callback. `Box<dyn FnOnce + Send + 'static>` itself satisfies
/// the generated `_then` methods' `impl FnOnce(…) + Send + 'static` bound, so the
/// channel-signalling callback can be built once and passed uniformly for every reducer.
type BoxedReducerCallback = Box<dyn FnOnce(&ReducerEventContext, ReducerOutcome) + Send + 'static>;

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
                    let _ = connect_tx.send(Ok(identity));
                }
            })
            .on_connect_error(move |_ctx, err| {
                let _ = connect_tx.send(Err(format!("{err:?}")));
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
                        self.conn
                            .reducers
                            .insert_message_then(id, sender, ROW_PAYLOAD.to_string(), cb)
                    })
                    .with_context(|| format!("seeding message id={id}"))?;
                }
                SeedOp::ChroniclePair { key, viewer } => {
                    self.await_reducer(|cb| {
                        self.conn
                            .reducers
                            .insert_chronicle_message_then(key, ROW_PAYLOAD.to_string(), cb)
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
        let (applied_tx, applied_rx) = mpsc::channel::<std::result::Result<(), String>>();
        self.conn
            .subscription_builder()
            .on_applied({
                let applied_tx = applied_tx.clone();
                move |_ctx| {
                    let _ = applied_tx.send(Ok(()));
                }
            })
            .on_error(move |_ctx, err| {
                let _ = applied_tx.send(Err(format!("{err:?}")));
            })
            .subscribe([target.subscription_sql()]);

        applied_rx
            .recv_timeout(SUBSCRIPTION_TIMEOUT)
            .context("waiting for the subscription to be applied")?
            .map_err(|msg| anyhow!("subscription failed: {msg}"))?;

        Ok(target.read_back(&self.conn))
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
            let _ = done_tx.send(flattened);
        });
        issue(callback).map_err(|e| anyhow!("issuing reducer: {e:?}"))?;
        done_rx
            .recv_timeout(REDUCER_TIMEOUT)
            .context("waiting for confirmed reducer completion")?
            .map_err(|msg| anyhow!("{msg}"))
    }
}
