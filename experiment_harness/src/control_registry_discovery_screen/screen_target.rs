//! What one attempt times.

use serde::Serialize;

use crate::client::connected_client::{
    TABLE_CONTROL_ACTIVITY, TABLE_CONTROL_ACTIVITY_LATEST_BY_CONTROL_VIEW, TABLE_CONTROL_REGISTRY,
    TABLE_CONTROL_REGISTRY_ALL_VIEW,
};
use crate::control_registry_discovery_screen::candidate_id::CandidateId;
use crate::control_registry_discovery_screen::screen_params::REGISTRY_CONTROLS;
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::plan::run_role::RunRole;

/// The four measured targets, one of which each attempt times cold.
///
/// Exactly one target is timed per attempt because E3 is a *cold* channel: a second subscription on
/// a fresh server is no longer cold, so four timings would need four servers regardless. All four
/// caches are nonetheless validated after the timed interval, which is what makes a composition
/// mismatch invalidate the pair.
///
/// The four are the `candidate × role` product, and that is why [`super::attempt_key::AttemptKey`]
/// needs no separate target component: `(ControlRegistry, Arm)` is Arm A, `(ControlRegistry,
/// Control)` its base-registry Control, and likewise for the comparator. Deriving the target from
/// identity rather than storing it alongside means the two can never disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenTarget {
    /// `control_registry_all_view` — the deployable O(K) registry arm.
    ArmA,
    /// Base `control_registry` — Arm A's composition Control.
    ControlA,
    /// `control_activity_latest_by_control_view` — the diagnostic O(N) comparator arm.
    ArmB,
    /// Base `control_activity` — Arm B's composition Control.
    ControlB,
}

impl ScreenTarget {
    /// Every target in canonical enumeration order.
    pub(crate) const ALL: [ScreenTarget; 4] = [
        ScreenTarget::ArmA,
        ScreenTarget::ControlA,
        ScreenTarget::ArmB,
        ScreenTarget::ControlB,
    ];

    /// The candidate this target produces evidence for.
    pub(crate) fn candidate(self) -> CandidateId {
        match self {
            Self::ArmA | Self::ControlA => CandidateId::ControlRegistry,
            Self::ArmB | Self::ControlB => CandidateId::ControlActivityLatestByControlView,
        }
    }

    /// Whether this target is the candidate under test or its matched composition Control.
    pub(crate) fn role(self) -> RunRole {
        match self {
            Self::ArmA | Self::ArmB => RunRole::Arm,
            Self::ControlA | Self::ControlB => RunRole::Control,
        }
    }

    /// Recover the target from an identity's `candidate × role` coordinate — the inverse of
    /// [`Self::candidate`] and [`Self::role`], so the driver reads one meaning out of the key it
    /// executes rather than carrying a second copy that could drift.
    pub(crate) fn of(candidate: CandidateId, role: RunRole) -> Self {
        match (candidate, role) {
            (CandidateId::ControlRegistry, RunRole::Arm) => Self::ArmA,
            (CandidateId::ControlRegistry, RunRole::Control) => Self::ControlA,
            (CandidateId::ControlActivityLatestByControlView, RunRole::Arm) => Self::ArmB,
            (CandidateId::ControlActivityLatestByControlView, RunRole::Control) => Self::ControlB,
        }
    }

    /// Rows this target's cache must hold once applied.
    ///
    /// Three of the four are `K` at both rungs — that invariance is the composition claim the screen
    /// checks, not an accident. Only the comparator's Control tracks `N`, because it subscribes to
    /// the append-only history itself.
    pub(crate) fn expected_rows(self, rung: ScreenRung) -> u64 {
        match self {
            Self::ArmA | Self::ControlA | Self::ArmB => REGISTRY_CONTROLS,
            Self::ControlB => rung.history_rows(),
        }
    }

    /// The subscription query this target is materialized by.
    ///
    /// **One source of truth for all four targets, timed and untimed alike.** The driver times
    /// exactly one target and then issues the other three as untimed validation subscriptions; both
    /// paths take their SQL from here, so the relation a cardinality is judged against is
    /// necessarily the relation that was subscribed. The table names are the existing
    /// `TABLE_CONTROL_*` constants — cross-component contracts with the module's own `#[view]` and
    /// `#[table]` accessors — rather than literals restated here.
    ///
    /// Copies [`MeasuredTarget::subscription_sql`](crate::view_read_set_campaign::measured_target::MeasuredTarget::subscription_sql),
    /// which is the same method for the campaign's two targets.
    pub(crate) fn subscription_sql(self) -> String {
        match self {
            Self::ArmA => format!("SELECT * FROM {TABLE_CONTROL_REGISTRY_ALL_VIEW}"),
            Self::ControlA => format!("SELECT * FROM {TABLE_CONTROL_REGISTRY}"),
            Self::ArmB => {
                format!("SELECT * FROM {TABLE_CONTROL_ACTIVITY_LATEST_BY_CONTROL_VIEW}")
            }
            Self::ControlB => format!("SELECT * FROM {TABLE_CONTROL_ACTIVITY}"),
        }
    }

    /// Stable tag naming this target in ledger records and order derivations. Spelled out rather
    /// than derived from the variant name, so a Rust rename cannot move a seeded order against a
    /// recorded seed.
    pub(crate) fn canonical_tag(self) -> &'static str {
        match self {
            Self::ArmA => "arm-a-control-registry-all-view",
            Self::ControlA => "control-a-base-control-registry",
            Self::ArmB => "arm-b-control-activity-latest-by-control-view",
            Self::ControlB => "control-b-base-control-activity",
        }
    }
}

#[cfg(test)]
mod tests;
