//! The two frozen endpoints this screen measures.

use serde::Serialize;

use crate::control_registry_discovery_screen::screen_params::{
    HIGH_RUNG_INDEX, LOW_RUNG_INDEX, REGISTRY_CONTROLS,
};
use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;

/// The screen's endpoint pair, as positions on the existing frozen unrelated/global ladder.
///
/// This type is the whole "ladder-only rungs" guarantee, and it is two guarantees stacked. A
/// [`GlobalRowRung`] can only be obtained from its own frozen `ALL` table, so no value off the
/// existing ladder can be constructed at all; and this enum has exactly two variants, so no ladder
/// position *other than* the two the freeze names can be reached from here. A rung is therefore
/// never a number this module chose — it is an index into a ladder frozen before this screen
/// existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum ScreenRung {
    /// 1,000 history rows — `GlobalRowRung(0)`.
    Low,
    /// 32,000 history rows — `GlobalRowRung(5)`.
    High,
}

impl ScreenRung {
    /// Both endpoints, low first. Not an execution order: [`super::screen_block_index`] decides
    /// that per block, and this is the canonical order identities are enumerated in.
    pub(crate) const ALL: [ScreenRung; 2] = [ScreenRung::Low, ScreenRung::High];

    /// This endpoint's position on the existing frozen ladder.
    pub(crate) fn global_rung(self) -> GlobalRowRung {
        match self {
            Self::Low => GlobalRowRung::ALL[LOW_RUNG_INDEX],
            Self::High => GlobalRowRung::ALL[HIGH_RUNG_INDEX],
        }
    }

    /// Total `control_activity` history rows seeded at this endpoint, `N`.
    pub(crate) fn history_rows(self) -> u64 {
        self.global_rung().global_rows()
    }

    /// History rows seeded per control, `N / K`. Exact at both endpoints by the compile-time
    /// divisibility proof in [`super::screen_params`], so this division never truncates.
    pub(crate) fn rows_per_control(self) -> u64 {
        self.history_rows() / REGISTRY_CONTROLS
    }

    /// Stable tag naming this rung in ledger records and order derivations. Built from the row count
    /// rather than the variant name, so a Rust rename cannot move a seeded order against a recorded
    /// seed.
    pub(crate) fn canonical_tag(self) -> String {
        format!("n{}", self.history_rows())
    }
}
