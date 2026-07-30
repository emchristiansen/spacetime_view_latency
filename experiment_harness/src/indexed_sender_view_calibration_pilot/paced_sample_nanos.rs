//! One paced append's raw latency, retained but never readable back as a number.

use serde::Serialize;

/// The raw nanoseconds of one paced append, from issue to the row becoming visible in the
/// subscriber's cache.
///
/// **This type is a visibility boundary**, and it carries part of the calibration ceiling. Holding a
/// bare `u128` in [`CalibrationSeries`](super::calibration_series::CalibrationSeries) would have
/// retained nothing privately — a `Vec<u128>` behind an accessor, or a public field, hands every
/// number back — so any code in this crate could reduce it to a median and report that as a cell
/// statistic without anyone noticing. A tuple struct with a private field in its **own module** is
/// the construct that stops the accessor from existing: sibling modules can hold and serialize these
/// values while being unable to read them directly.
///
/// **What this does and does not guarantee — stated exactly, because the difference matters.** No
/// accessor exists and no field access compiles outside this module, so there is no *direct* path
/// from a recorded sample to arithmetic, and no reduction API can be added by accident. That is the
/// whole of it. `Serialize` is required — retaining every ordered raw nanosecond in the ledger is
/// the entire purpose of the pilot — and a serde round trip inside this crate would recover the
/// numbers. **This is therefore not a capability boundary and must not be described as one.** A
/// determined caller can serialize and re-parse; nothing here prevents that.
///
/// What actually prevents a performance claim is the record vocabulary plus review: there is no
/// outcome variant, no evidence type, and no ledger field shaped like a result, so a reduction
/// computed by that route would have nowhere honest to go and would be visible in review as new
/// parsing code written for no stated purpose. The types remove the easy and the accidental path;
/// the ceiling itself is enforced by what the ledger can say.
///
/// **Neither `Debug` nor `PartialEq` nor `Ord` is derived**, for the same reason: each would add a
/// convenient recovery route — textual rendering, recovery by comparison search, a reachable minimum
/// or maximum — that no legitimate use of this type needs.
///
/// Construction is deliberately unrestricted, including of a zero: minting one is harmless, because
/// a `PacedSampleNanos` can do nothing except be recorded, and a nonpositive sample is exactly the
/// thing a calibration ledger must show rather than silently drop.
#[derive(Clone, Copy, Serialize)]
#[serde(transparent)]
pub(crate) struct PacedSampleNanos(u128);

impl PacedSampleNanos {
    /// Retain one paced append's raw latency.
    pub(crate) fn of(nanos: u128) -> Self {
        Self(nanos)
    }

    /// Whether this sample is strictly positive.
    ///
    /// The one predicate this module exposes, and it answers a yes/no question rather than handing
    /// back the number. [`CalibrationSeries`](super::calibration_series::CalibrationSeries) needs it
    /// because the spec invalidates a cell on a nonpositive statistic, so a series containing one
    /// cannot be recorded as usable — but deciding that must not require the value to escape.
    pub(crate) fn is_positive(self) -> bool {
        self.0 > 0
    }
}
