//! The single frozen endpoint this pilot runs at.

use serde::Serialize;

use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;
use crate::indexed_sender_view_calibration_pilot::calibration_params::BASELINE_RUNG_INDEX;

/// The pilot's endpoint, as a position on the existing frozen unrelated/global ladder.
///
/// Two guarantees stacked, exactly as the discovery screen's rung type stacks them. A
/// [`GlobalRowRung`] can only be obtained from its own frozen `ALL` table, so no value off the
/// existing ladder is constructible at all; and this enum has exactly one variant, so no ladder
/// position other than the baseline the spec's decision names can be reached from here. A rung is
/// therefore never a number this module chose.
///
/// One variant rather than two is the whole methodological point: a second rung would make the two
/// attempts a low/high contrast, which is precisely the comparison the calibration ceiling forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum CalibrationRung {
    /// 1,000 unrelated rows — `GlobalRowRung(0)`, the governing unrelated/global baseline.
    Baseline,
}

impl CalibrationRung {
    /// Every rung this pilot runs at.
    pub(crate) const ALL: [CalibrationRung; 1] = [CalibrationRung::Baseline];

    /// This endpoint's position on the existing frozen ladder.
    pub(crate) fn global_rung(self) -> GlobalRowRung {
        match self {
            Self::Baseline => GlobalRowRung::ALL[BASELINE_RUNG_INDEX],
        }
    }

    /// Unrelated `indexed_control_activity` rows seeded at this endpoint, owned by the
    /// non-connecting other owner — `N`.
    pub(crate) fn unrelated_rows(self) -> u64 {
        self.global_rung().global_rows()
    }

    /// Stable tag naming this rung in ledger records. Built from the row count rather than the
    /// variant name, so a Rust rename cannot move a recorded tag.
    pub(crate) fn canonical_tag(self) -> String {
        format!("n{}", self.unrelated_rows())
    }
}
