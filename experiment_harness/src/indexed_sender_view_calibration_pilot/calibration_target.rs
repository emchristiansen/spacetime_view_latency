//! The relations one calibration attempt subscribes to.

use serde::Serialize;

use crate::client::connected_client::{
    TABLE_INDEXED_CONTROL_ACTIVITY, TABLE_INDEXED_CONTROL_ACTIVITY_SENDER_VIEW,
};

/// The two relations an attempt subscribes to, and the sharply different jobs they do.
///
/// **This is not an Arm/Control pair, and the distinction is load-bearing.** The spec's calibration
/// ceiling forbids an Arm/Control comparison, and none is possible here: only
/// [`Self::MeasuredArm`] is ever timed, [`Self::CompositionWitness`] carries no channel and no
/// sample, and no type in this module holds two latencies to divide. The witness exists for one
/// reason — to establish that the unrelated population was actually seeded, so a series recorded
/// against an empty backing table cannot be mistaken for one recorded at the baseline rung.
///
/// The witness is subscribed **after** the paced batch completes, never before, so it adds no
/// subscription to the connection during the measured window and cannot perturb what the pilot is
/// calibrating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum CalibrationTarget {
    /// `indexed_control_activity_sender_view` — the candidate arm, and the only timed relation.
    MeasuredArm,
    /// Base `indexed_control_activity` — the untimed composition witness.
    CompositionWitness,
}

impl CalibrationTarget {
    /// The subscription query this target is materialized by.
    ///
    /// One source of truth for both, timed and untimed alike, so the relation a cardinality is
    /// judged against is necessarily the relation that was subscribed. The table names are the
    /// existing `TABLE_INDEXED_CONTROL_ACTIVITY*` constants — cross-component contracts with the
    /// module's own `#[view]` and `#[table]` accessors — rather than literals restated here.
    pub(crate) fn subscription_sql(self) -> String {
        match self {
            Self::MeasuredArm => {
                format!("SELECT * FROM {TABLE_INDEXED_CONTROL_ACTIVITY_SENDER_VIEW}")
            }
            Self::CompositionWitness => format!("SELECT * FROM {TABLE_INDEXED_CONTROL_ACTIVITY}"),
        }
    }

    /// Stable tag naming this target in ledger records. Spelled out rather than derived from the
    /// variant name, so a Rust rename cannot move a recorded tag.
    pub(crate) fn canonical_tag(self) -> &'static str {
        match self {
            Self::MeasuredArm => "arm-indexed-control-activity-sender-view",
            Self::CompositionWitness => "witness-base-indexed-control-activity",
        }
    }
}
