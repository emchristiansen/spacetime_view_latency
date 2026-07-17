//! The single table a run subscribes to, and how its rows are named, expected, and read.

use std::sync::Arc;

#[rustfmt::skip]
#[allow(unused_imports)]
use crate::module_artifact::bindings::*;
use spacetimedb_sdk::Table;
use spacetimedb_sdk::TableWithPrimaryKey;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::seed_plan::SeedPlan;
use crate::dataset::subscribed_rows::SubscribedRows;
use crate::observation::dose_event_counter::DoseEventCounter;
use crate::plan::cell::Cell;
use crate::plan::control_table::ControlTable;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::plan::run::Run;
use crate::plan::run_role::RunRole;
use crate::plan::table_scoped_arm::TableScopedArm;

// Subscribable table names. Cross-component contracts that must match the module's
// `#[table(accessor = …)]` / `#[view(accessor = …)]` names and the generated client
// accessors, so they are named constants, never inline literals.
const TABLE_MESSAGE: &str = "message";
const TABLE_MESSAGE_RANGE_VIEW: &str = "message_range_view";
const TABLE_MESSAGE_QUERY_VIEW: &str = "message_query_view";
const TABLE_MESSAGE_QUERY_PK_VIEW: &str = "message_query_pk_view";
const TABLE_MESSAGES_POINT_VIEW: &str = "messages_point_view";
const TABLE_CHRONICLE_MESSAGE: &str = "chronicle_message";
const TABLE_CHRONICLE_QUERY_VIEW: &str = "chronicle_query_view";
const TABLE_CHRONICLE_QUERY_PK_VIEW: &str = "chronicle_query_pk_view";
const TABLE_CHRONICLE_POINT_VIEW: &str = "chronicle_point_view";

/// Whether the subscribed table returns the full base-table dataset or only the measured
/// identity's own result slice. This distinguishes full-pass-through arms (and every
/// direct-table control) from the identity-filtered arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// The full dataset: measured slice plus growth slice.
    FullTable,
    /// Only the measured identity's own result slice.
    MeasuredSlice,
}

/// Which of the module's subscribable tables a run subscribes to. This is the single
/// Run→target decision: the subscription SQL, the row family and scope used to build the
/// expected set, and the cache read-back all derive from it, so they cannot drift apart.
#[derive(Debug, Clone, Copy)]
pub(crate) enum SubscribedTable {
    /// Direct `message` base-table control.
    MessageBase,
    /// Arm A — procedural full-domain range view.
    MessageRangeView,
    /// Arm B — query-builder full pass-through view.
    MessageQueryView,
    /// Arm D — B plus custom view primary key.
    MessageQueryPkView,
    /// Arm F — procedural point-filter view.
    MessagesPointView,
    /// Direct `chronicle_message` base-table control.
    ChronicleBase,
    /// Arm C — query-builder semijoin view.
    ChronicleQueryView,
    /// Arm E — C plus custom view primary key.
    ChronicleQueryPkView,
    /// Arm F′ — procedural hand-written semijoin view.
    ChroniclePointView,
}

impl SubscribedTable {
    /// The table a run subscribes to: an arm run subscribes to its module view; a control
    /// run subscribes directly to its matched base table.
    pub(crate) fn from_run(run: Run) -> Self {
        match run.role() {
            RunRole::Control => match run.control_table() {
                ControlTable::Message => SubscribedTable::MessageBase,
                ControlTable::ChronicleMessage => SubscribedTable::ChronicleBase,
            },
            RunRole::Arm => match run.cell() {
                Cell::TableScopedUnrelated(arm) => match arm {
                    TableScopedArm::ProceduralRange => SubscribedTable::MessageRangeView,
                    TableScopedArm::QueryFull => SubscribedTable::MessageQueryView,
                    TableScopedArm::QuerySemijoin => SubscribedTable::ChronicleQueryView,
                    TableScopedArm::QueryFullPk => SubscribedTable::MessageQueryPkView,
                    TableScopedArm::QuerySemijoinPk => SubscribedTable::ChronicleQueryPkView,
                },
                Cell::KeyScopedUnrelated(arm) | Cell::KeyScopedOwnSlice(arm) => match arm {
                    KeyScopedArm::PointFilter => SubscribedTable::MessagesPointView,
                    KeyScopedArm::PointSemijoin => SubscribedTable::ChroniclePointView,
                },
            },
        }
    }

    /// The subscribed table name.
    fn table_name(self) -> &'static str {
        match self {
            SubscribedTable::MessageBase => TABLE_MESSAGE,
            SubscribedTable::MessageRangeView => TABLE_MESSAGE_RANGE_VIEW,
            SubscribedTable::MessageQueryView => TABLE_MESSAGE_QUERY_VIEW,
            SubscribedTable::MessageQueryPkView => TABLE_MESSAGE_QUERY_PK_VIEW,
            SubscribedTable::MessagesPointView => TABLE_MESSAGES_POINT_VIEW,
            SubscribedTable::ChronicleBase => TABLE_CHRONICLE_MESSAGE,
            SubscribedTable::ChronicleQueryView => TABLE_CHRONICLE_QUERY_VIEW,
            SubscribedTable::ChronicleQueryPkView => TABLE_CHRONICLE_QUERY_PK_VIEW,
            SubscribedTable::ChroniclePointView => TABLE_CHRONICLE_POINT_VIEW,
        }
    }

    /// The `SELECT * FROM <table>` subscription query for this target.
    pub(crate) fn subscription_sql(self) -> String {
        format!("SELECT * FROM {}", self.table_name())
    }

    /// The row family (`message` vs `chronicle_message`) of the subscribed rows.
    fn row_family(self) -> ControlTable {
        match self {
            SubscribedTable::MessageBase
            | SubscribedTable::MessageRangeView
            | SubscribedTable::MessageQueryView
            | SubscribedTable::MessageQueryPkView
            | SubscribedTable::MessagesPointView => ControlTable::Message,
            SubscribedTable::ChronicleBase
            | SubscribedTable::ChronicleQueryView
            | SubscribedTable::ChronicleQueryPkView
            | SubscribedTable::ChroniclePointView => ControlTable::ChronicleMessage,
        }
    }

    /// Whether this target returns the full dataset or only the measured slice.
    fn scope(self) -> Scope {
        match self {
            SubscribedTable::MessageBase
            | SubscribedTable::ChronicleBase
            | SubscribedTable::MessageRangeView
            | SubscribedTable::MessageQueryView
            | SubscribedTable::MessageQueryPkView => Scope::FullTable,
            SubscribedTable::MessagesPointView
            | SubscribedTable::ChronicleQueryView
            | SubscribedTable::ChronicleQueryPkView
            | SubscribedTable::ChroniclePointView => Scope::MeasuredSlice,
        }
    }

    /// The seed-derived expected result set for this target: the measured slice, plus the
    /// growth slice when the target returns the full dataset. Every C/E/F′ arm has
    /// `MeasuredSlice` scope over the Chronicle family, so all three expect exactly the
    /// measured slice's Chronicle rows — the transitive equivalence certification.
    pub(crate) fn expected(self, plan: &SeedPlan) -> SubscribedRows {
        let measured = plan.measured();
        let growth = plan.growth();
        match self.row_family() {
            ControlTable::Message => {
                let mut rows = measured.expected_messages();
                if self.scope() == Scope::FullTable {
                    rows.extend(growth.expected_messages());
                }
                SubscribedRows::Message(rows)
            }
            ControlTable::ChronicleMessage => {
                let mut rows = measured.expected_chronicle();
                if self.scope() == Scope::FullTable {
                    rows.extend(growth.expected_chronicle());
                }
                SubscribedRows::Chronicle(rows)
            }
        }
    }

    /// The cumulative expected result set for this target once the doses `1..=through` have been applied
    /// on top of the unmeasured pinned background slice — the post-dose analogue of [`Self::expected`]'s
    /// single-rung seed set. Derived from the resolved [`CampaignDataset`] (its family, pinned baseline,
    /// and driving key space) and this target's own [`Scope`], never from key arithmetic reconstructed in
    /// the driver: a [`Scope::FullTable`] target grows by every completed driving batch, while a
    /// [`Scope::MeasuredSlice`] target grows only when the driving role *is* its own measured slice (the
    /// `OwnSliceGrowth` regime) and otherwise holds its pinned slice constant across the ladder.
    pub(crate) fn expected_through_dose(
        self,
        dataset: &CampaignDataset,
        through: DoseIndex,
    ) -> SubscribedRows {
        let measured = dataset.measured_slice_through(through);
        let growth = dataset.growth_slice_through(through);
        match self.row_family() {
            ControlTable::Message => {
                let mut rows = measured.expected_messages();
                if self.scope() == Scope::FullTable {
                    rows.extend(growth.expected_messages());
                }
                SubscribedRows::Message(rows)
            }
            ControlTable::ChronicleMessage => {
                let mut rows = measured.expected_chronicle();
                if self.scope() == Scope::FullTable {
                    rows.extend(growth.expected_chronicle());
                }
                SubscribedRows::Chronicle(rows)
            }
        }
    }

    /// The seed-derived expected result set for this target *before any dose*, once only the
    /// unmeasured pinned background slice has been applied — the pre-dose baseline the initial-set
    /// check asserts. The cumulative-ladder analogue of [`Self::expected`] at zero applied doses:
    /// derived from the resolved [`CampaignDataset`]'s initial slices (the driving role contributes
    /// zero rows, the pinned role its held-constant baseline) and this target's [`Scope`], never from
    /// key arithmetic reconstructed in the driver. Identical scope/family projection to
    /// [`Self::expected_through_dose`], only over the initial slices.
    pub(crate) fn expected_initial(self, dataset: &CampaignDataset) -> SubscribedRows {
        let measured = dataset.measured_slice_initial();
        let growth = dataset.growth_slice_initial();
        match self.row_family() {
            ControlTable::Message => {
                let mut rows = measured.expected_messages();
                if self.scope() == Scope::FullTable {
                    rows.extend(growth.expected_messages());
                }
                SubscribedRows::Message(rows)
            }
            ControlTable::ChronicleMessage => {
                let mut rows = measured.expected_chronicle();
                if self.scope() == Scope::FullTable {
                    rows.extend(growth.expected_chronicle());
                }
                SubscribedRows::Chronicle(rows)
            }
        }
    }

    /// Register the per-dose SDK row-event callbacks for this target's cache table, each incrementing
    /// the shared [`DoseEventCounter`] as the SDK delivers a client-visible insert/delete/update. The
    /// same per-variant table dispatch as [`Self::read_back`], so the counted table can never drift
    /// from the subscribed one. Insert and delete are registered for every target; **update** is
    /// registered only for the targets whose generated handle implements
    /// [`TableWithPrimaryKey`](spacetimedb_sdk::TableWithPrimaryKey) (the primary-keyed base tables and
    /// query views) — the primary-key-less procedural range/point views deliver only insert/delete, and
    /// their handles have no `on_update`, so a spurious update registration is unrepresentable rather
    /// than merely unused. The returned callback ids are intentionally dropped: the SDK removes a
    /// callback only on an explicit `remove_on_*`, so the callbacks persist for the run's lifetime.
    ///
    /// Registered by the driver only **after** the subscription's `on_applied` snapshot, the
    /// initial-set verification, and the unmeasured warm-up — the warm-up is seeded *before* the
    /// subscription, so its rows arrive in the snapshot rather than as incremental events, and are
    /// excluded along with the rest of the snapshot. So neither snapshot nor warm-up inserts are
    /// counted toward any dose (see
    /// [`EventEvidence`](crate::observation::event_evidence::EventEvidence)).
    pub(crate) fn register_events(self, conn: &DbConnection, counter: &Arc<DoseEventCounter>) {
        match self {
            SubscribedTable::MessageBase => {
                register_insert_delete(&conn.db.message(), counter);
                register_update(&conn.db.message(), counter);
            }
            SubscribedTable::MessageRangeView => {
                register_insert_delete(&conn.db.message_range_view(), counter);
            }
            SubscribedTable::MessageQueryView => {
                register_insert_delete(&conn.db.message_query_view(), counter);
                register_update(&conn.db.message_query_view(), counter);
            }
            SubscribedTable::MessageQueryPkView => {
                register_insert_delete(&conn.db.message_query_pk_view(), counter);
                register_update(&conn.db.message_query_pk_view(), counter);
            }
            SubscribedTable::MessagesPointView => {
                register_insert_delete(&conn.db.messages_point_view(), counter);
            }
            SubscribedTable::ChronicleBase => {
                register_insert_delete(&conn.db.chronicle_message(), counter);
                register_update(&conn.db.chronicle_message(), counter);
            }
            SubscribedTable::ChronicleQueryView => {
                register_insert_delete(&conn.db.chronicle_query_view(), counter);
                register_update(&conn.db.chronicle_query_view(), counter);
            }
            SubscribedTable::ChronicleQueryPkView => {
                register_insert_delete(&conn.db.chronicle_query_pk_view(), counter);
                register_update(&conn.db.chronicle_query_pk_view(), counter);
            }
            SubscribedTable::ChroniclePointView => {
                register_insert_delete(&conn.db.chronicle_point_view(), counter);
            }
        }
    }

    /// Read this target's currently-subscribed rows out of the client cache.
    pub(crate) fn read_back(self, conn: &DbConnection) -> SubscribedRows {
        match self {
            SubscribedTable::MessageBase => {
                SubscribedRows::Message(conn.db.message().iter().collect())
            }
            SubscribedTable::MessageRangeView => {
                SubscribedRows::Message(conn.db.message_range_view().iter().collect())
            }
            SubscribedTable::MessageQueryView => {
                SubscribedRows::Message(conn.db.message_query_view().iter().collect())
            }
            SubscribedTable::MessageQueryPkView => {
                SubscribedRows::Message(conn.db.message_query_pk_view().iter().collect())
            }
            SubscribedTable::MessagesPointView => {
                SubscribedRows::Message(conn.db.messages_point_view().iter().collect())
            }
            SubscribedTable::ChronicleBase => {
                SubscribedRows::Chronicle(conn.db.chronicle_message().iter().collect())
            }
            SubscribedTable::ChronicleQueryView => {
                SubscribedRows::Chronicle(conn.db.chronicle_query_view().iter().collect())
            }
            SubscribedTable::ChronicleQueryPkView => {
                SubscribedRows::Chronicle(conn.db.chronicle_query_pk_view().iter().collect())
            }
            SubscribedTable::ChroniclePointView => {
                SubscribedRows::Chronicle(conn.db.chronicle_point_view().iter().collect())
            }
        }
    }
}

/// Register insert and delete counter callbacks on one cache table handle. Generic over any
/// [`Table`] so the same body serves every target; each callback owns its own [`Arc`] clone of the
/// shared counter and ignores the delivered context/row — only the event's occurrence is counted.
fn register_insert_delete<T: Table>(table: &T, counter: &Arc<DoseEventCounter>) {
    table.on_insert({
        let counter = counter.clone();
        move |_ctx, _row| counter.record_insert()
    });
    table.on_delete({
        let counter = counter.clone();
        move |_ctx, _row| counter.record_delete()
    });
}

/// Register the update counter callback on one primary-keyed cache table handle. Generic over
/// [`TableWithPrimaryKey`] so only the targets whose generated handle actually delivers updates can be
/// passed — the primary-key-less views have no such handle and cannot reach this function.
fn register_update<T: TableWithPrimaryKey>(table: &T, counter: &Arc<DoseEventCounter>) {
    table.on_update({
        let counter = counter.clone();
        move |_ctx, _old, _new| counter.record_update()
    });
}

#[cfg(test)]
mod tests;
