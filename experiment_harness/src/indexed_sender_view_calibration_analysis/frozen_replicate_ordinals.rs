//! The exact replicate ordinals the frozen inventory contains.

use std::collections::BTreeSet;

use crate::indexed_sender_view_calibration_pilot::calibration_replicate::CalibrationReplicate;

/// Every replicate ordinal the freeze declares, derived from the pilot's own
/// [`CalibrationReplicate::ALL`] rather than restated as `{0, 1}`.
///
/// **Distinctness is not the requirement; membership is.** Two admitted records at ordinals 2 and 3
/// are distinct, and pairing them would silently promote two records the frozen inventory never
/// declared into "the two originals". The inventory names exactly which slots exist, so the pairing
/// check is set equality against that, not a count plus an inequality.
///
/// Derived rather than written out so a change to `CALIBRATION_ATTEMPTS` moves this set with it. The
/// pilot's inventory seal and this reader then cannot disagree about which slots the run was
/// supposed to fill.
pub(crate) fn frozen_replicate_ordinals() -> BTreeSet<u32> {
    CalibrationReplicate::ALL
        .iter()
        .map(|replicate| replicate.get())
        .collect()
}
