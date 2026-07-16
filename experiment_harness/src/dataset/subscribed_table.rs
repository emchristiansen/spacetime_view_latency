//! The single table a run subscribes to, and how its rows are named, expected, and read.

#[rustfmt::skip]
#[allow(unused_imports)]
use crate::module_artifact::bindings::*;
use spacetimedb_sdk::Table;

use crate::dataset::seed_plan::SeedPlan;
use crate::dataset::subscribed_rows::SubscribedRows;
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

    /// Read this target's currently-subscribed rows out of the client cache.
    pub(crate) fn read_back(self, conn: &DbConnection) -> SubscribedRows {
        match self {
            SubscribedTable::MessageBase => SubscribedRows::Message(conn.db.message().iter().collect()),
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
